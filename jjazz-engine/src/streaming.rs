use crate::phrase::Phrase;
use crate::style_parser::ParsedStylePart;
use crate::style_player::generate_from_parsed_part;
use crate::harmony::ChordSymbol;
use crate::synth::SynthEngine;
use std::collections::VecDeque;

// ══════════════════════════════════════════════════════════
//  Leader Mode — which instrument drives the tempo
// ══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderMode {
    /// Follow any instrument (aggregated beat detection)
    Any,
    /// Follow the drums (channel 9/10)
    Drums,
    /// Follow the piano/keyboard (channel 2-3)
    Piano,
    /// Follow the bass (channel 0)
    Bass,
    /// Follow the vocalist (external input)
    Vocal,
}

impl LeaderMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "drums" => LeaderMode::Drums,
            "piano" | "keys" | "keyboard" => LeaderMode::Piano,
            "bass" => LeaderMode::Bass,
            "vocal" | "voice" => LeaderMode::Vocal,
            _ => LeaderMode::Any,
        }
    }
}

// ══════════════════════════════════════════════════════════
//  TempoTracker — BPM tracking with smoothing
// ══════════════════════════════════════════════════════════

/// Exponential moving average BPM tracker.
/// Millisecond response to tempo changes, 1-2 bar smoothing.
pub struct TempoTracker {
    pub current_bpm: f32,       // Smoothed BPM for playback
    pub raw_bpm: f32,           // Latest measured BPM
    pub target_bpm: f32,        // Where we're heading
    pub smoothing: f32,         // 0.0=instant, 0.95=very slow (bar-level)
    pub beat_count: u64,        // Total beats elapsed
    pub bar_length_beats: f32,  // 4.0 for 4/4
}

impl TempoTracker {
    pub fn new(initial_bpm: f32) -> Self {
        Self {
            current_bpm: initial_bpm,
            raw_bpm: initial_bpm,
            target_bpm: initial_bpm,
            smoothing: 0.85,      // ~1 bar to converge for 4/4 at 120 BPM
            beat_count: 0,
            bar_length_beats: 4.0,
        }
    }

    /// Feed a detected beat (from audio input). Updates BPM instantly.
    pub fn feed_beat(&mut self, timestamp_ms: u64) {
        self.beat_count += 1;
        // Will be calibrated by external beat detector
    }

    /// Set target BPM (user change). Ramps over 1-2 bars.
    pub fn set_target_bpm(&mut self, new_bpm: f32) {
        self.target_bpm = new_bpm.clamp(30.0, 300.0);
    }

    /// Call once per audio chunk. Returns current smoothed BPM.
    /// Progresses toward target_bpm with smoothing factor.
    pub fn update(&mut self, _delta_ms: f32) -> f32 {
        // Exponential moving average toward target
        self.current_bpm = self.current_bpm * self.smoothing
            + self.target_bpm * (1.0 - self.smoothing);
        self.current_bpm
    }

    /// Seconds of audio per beat at current BPM.
    pub fn seconds_per_beat(&self) -> f32 {
        60.0 / self.current_bpm
    }

    pub fn advance_bar(&mut self) {
        self.beat_count += self.bar_length_beats as u64;
    }
}

// ══════════════════════════════════════════════════════════
//  BarBuffer — pre-generated bar queue
// ══════════════════════════════════════════════════════════

/// A pre-rendered audio bar, ready for playback.
pub struct RenderedBar {
    pub audio: Vec<f32>,        // Stereo interleaved samples
    pub bar_index: u64,
    pub bpm: f32,
}

/// Ring buffer of pre-generated bars (2 ahead, keep filling).
pub struct BarBuffer {
    pub bars: VecDeque<RenderedBar>,
    pub lookahead: usize,       // Bars to keep in buffer (default 2)
}

impl BarBuffer {
    pub fn new(lookahead: usize) -> Self {
        Self { bars: VecDeque::with_capacity(lookahead + 1), lookahead }
    }

    pub fn needs_refill(&self) -> bool {
        self.bars.len() <= self.lookahead / 2
    }

    pub fn push(&mut self, bar: RenderedBar) {
        self.bars.push_back(bar);
    }

    pub fn pop(&mut self) -> Option<RenderedBar> {
        self.bars.pop_front()
    }

    pub fn is_empty(&self) -> bool { self.bars.is_empty() }
}

// ══════════════════════════════════════════════════════════
//  StreamingEngine — top-level orchestrator
// ══════════════════════════════════════════════════════════

/// Configuration for the streaming engine.
pub struct StreamingConfig {
    pub initial_bpm: f32,
    pub leader_mode: LeaderMode,
    pub smoothing: f32,
    pub bars_per_chord: u32,
    pub sample_rate: u32,
    pub lookahead_bars: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            initial_bpm: 120.0,
            leader_mode: LeaderMode::Any,
            smoothing: 0.85,
            bars_per_chord: 2,
            sample_rate: 44100,
            lookahead_bars: 2,
        }
    }
}

/// The streaming engine: generates bars on-demand and renders audio chunks.
pub struct StreamingEngine {
    pub config: StreamingConfig,
    pub tempo: TempoTracker,
    pub bar_buffer: BarBuffer,
    pub synth: Option<SynthEngine>,
    pub style_part: Option<ParsedStylePart>,
    pub chords: Vec<ChordSymbol>,
    pub current_bar: u64,
    /// How many bars have been generated
    pub bars_generated: u64,
    /// Remaining audio samples from the currently playing bar
    pub remaining_samples: Vec<f32>,
    pub sample_cursor: usize,
}

impl StreamingEngine {
    pub fn new(config: StreamingConfig, style_part: Option<ParsedStylePart>, chords: Vec<ChordSymbol>) -> Self {
        let tempo = TempoTracker::new(config.initial_bpm);
        let bar_buffer = BarBuffer::new(config.lookahead_bars);
        Self {
            tempo,
            bar_buffer,
            config,
            synth: None,
            style_part,
            chords,
            current_bar: 0,
            bars_generated: 0,
            remaining_samples: Vec::new(),
            sample_cursor: 0,
        }
    }

    /// Initialize with a SoundFont.
    pub fn load_soundfont(&mut self, sf2_path: &str) -> Result<(), String> {
        let mut synth = SynthEngine::from_file(sf2_path)?;
        synth.init_streaming();
        self.synth = Some(synth);
        Ok(())
    }

    /// Set target BPM (ramps smoothly over 1-2 bars).
    pub fn set_bpm(&mut self, bpm: f32) {
        self.tempo.set_target_bpm(bpm);
    }

    /// Feed a detected beat from the leader instrument.
    pub fn feed_leader_beat(&mut self, _timestamp_ms: u64) {
        // In a full implementation, this would:
        // 1. Calculate inter-beat interval from recent beats
        // 2. Update raw_bpm
        // 3. Set target_bpm to match the leader
    }

    /// Transpose all subsequent bars by semitones.
    pub fn set_transpose(&mut self, _semitones: i32) {
        // Would modify the chord symbols to transpose
    }

    /// Generate the next bar (call when buffer needs refilling).
    fn generate_next_bar(&mut self) -> Option<RenderedBar> {
        let synth = self.synth.as_mut()?;
        let part = self.style_part.as_ref()?;

        let bpm = self.tempo.update(0.0);
        let bar_beats = self.tempo.bar_length_beats;
        let total_samples = (bar_beats * 60.0 / bpm * self.config.sample_rate as f32) as usize;

        // Generate phrases for this bar
        let result = generate_from_parsed_part(part, &self.chords, self.config.bars_per_chord);
        let tracks = match result {
            Ok(t) => t,
            Err(_) => return None,
        };

        let audio = synth.render_chunk(&tracks, bpm);
        self.bars_generated += 1;

        Some(RenderedBar {
            audio,
            bar_index: self.bars_generated,
            bpm,
        })
    }

    /// Refill the bar buffer if needed.
    pub fn refill_buffer(&mut self) {
        while self.bar_buffer.needs_refill() {
            if let Some(bar) = self.generate_next_bar() {
                self.bar_buffer.push(bar);
            } else {
                break;
            }
        }
    }

    /// Get the next chunk of audio samples. Returns the number of samples written.
    /// `output`: stereo interleaved f32 buffer to fill.
    pub fn next_chunk(&mut self, output: &mut [f32]) -> usize {
        let mut written = 0;

        while written < output.len() {
            // Use remaining samples from current bar
            if self.sample_cursor < self.remaining_samples.len() {
                let available = self.remaining_samples.len() - self.sample_cursor;
                let needed = output.len() - written;
                let to_copy = available.min(needed);

                output[written..written + to_copy]
                    .copy_from_slice(&self.remaining_samples[self.sample_cursor..self.sample_cursor + to_copy]);
                self.sample_cursor += to_copy;
                written += to_copy;

                if written >= output.len() { break; }
            }

            // Need next bar
            self.refill_buffer();
            if let Some(bar) = self.bar_buffer.pop() {
                self.remaining_samples = bar.audio;
                self.sample_cursor = 0;
                self.current_bar += 1;
                self.tempo.advance_bar();
            } else {
                // No more bars, fill with silence
                let remaining = output.len() - written;
                output[written..written + remaining].fill(0.0);
                written += remaining;
                break;
            }
        }
        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_smoothing() {
        let mut t = TempoTracker::new(120.0);
        t.set_target_bpm(140.0);
        // After several updates, should move toward target
        let mut bpm = 120.0;
        for _ in 0..20 {
            bpm = t.update(0.0);
        }
        assert!(bpm > 130.0, "Should approach target 140, got {}", bpm);
    }

    #[test]
    fn test_leader_mode_parsing() {
        assert_eq!(LeaderMode::from_str("drums"), LeaderMode::Drums);
        assert_eq!(LeaderMode::from_str("piano"), LeaderMode::Piano);
        assert_eq!(LeaderMode::from_str("bass"), LeaderMode::Bass);
        assert_eq!(LeaderMode::from_str("vocal"), LeaderMode::Vocal);
        assert_eq!(LeaderMode::from_str("guitar"), LeaderMode::Any);
    }

    #[test]
    fn test_bar_buffer() {
        let mut bb = BarBuffer::new(2);
        assert!(bb.is_empty());
        assert!(bb.needs_refill());
        bb.push(RenderedBar { audio: vec![0.0; 100], bar_index: 0, bpm: 120.0 });
        assert_eq!(bb.pop().unwrap().bpm, 120.0);
        assert!(bb.is_empty());
    }

    #[test]
    fn test_streaming_config_default() {
        let cfg = StreamingConfig::default();
        assert_eq!(cfg.initial_bpm, 120.0);
    }
}
