use crate::harmony::{ChordSymbol, Note};
use crate::harmony::chord_types::TYPES;
use std::collections::VecDeque;

// ══════════════════════════════════════════════════════════
//  Pitch Detection: autocorrelation on f32 audio buffer
// ══════════════════════════════════════════════════════════

/// Detect the fundamental frequency (Hz) from an audio buffer using autocorrelation.
/// Returns None if no clear pitch is detected.
pub fn detect_pitch(samples: &[f32], sample_rate: u32) -> Option<f32> {
    if samples.len() < 256 { return None; }

    // Compute RMS to check if there's actual signal
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    if rms < 0.005 { return None; } // silence threshold

    // Autocorrelation: find the lag with maximum correlation
    let min_lag = (sample_rate as f32 / 1200.0) as usize; // ~37 Hz = low C
    let max_lag = (sample_rate as f32 / 60.0) as usize;   // ~1000 Hz = high C
    let n = samples.len();

    let mut best_lag = 0;
    let mut best_corr = -1.0f32;

    for lag in min_lag..max_lag.min(n / 2) {
        let mut corr = 0.0f32;
        for i in 0..n - lag {
            corr += samples[i] * samples[i + lag];
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    if best_corr < 0.01 { return None; }

    let freq = sample_rate as f32 / best_lag as f32;
    if freq < 50.0 || freq > 1200.0 { return None; }
    Some(freq)
}

/// Convert frequency (Hz) to MIDI note number.
pub fn freq_to_midi(freq: f32) -> u8 {
    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    midi.round().clamp(0.0, 127.0) as u8
}

/// Convert MIDI note number to note name (e.g., "C4").
pub fn midi_to_name(midi: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi as i32 / 12) - 1;
    let name = names[(midi % 12) as usize];
    format!("{}{}", name, octave)
}

// ══════════════════════════════════════════════════════════
//  Chord Recognition: detected pitches → ChordSymbol
// ══════════════════════════════════════════════════════════

/// A buffer of recently detected pitches for chord recognition.
pub struct PitchBuffer {
    pub notes: VecDeque<(u8, f32)>, // (midi_note, timestamp_sec)
    pub window_secs: f32,
}

impl PitchBuffer {
    pub fn new(window_secs: f32) -> Self {
        Self { notes: VecDeque::new(), window_secs }
    }

    /// Add a detected pitch with timestamp.
    pub fn push(&mut self, midi: u8, timestamp_sec: f32) {
        self.notes.push_back((midi, timestamp_sec));
        // Remove old notes outside the window
        let cutoff = timestamp_sec - self.window_secs;
        while self.notes.front().map_or(false, |(_, t)| *t < cutoff) {
            self.notes.pop_front();
        }
    }

    /// Get unique MIDI notes currently in the buffer.
    pub fn unique_pitches(&self) -> Vec<u8> {
        let mut pitches: Vec<u8> = self.notes.iter().map(|(m, _)| *m).collect();
        pitches.sort_unstable();
        pitches.dedup();
        pitches
    }

    /// Try to recognize a chord from the buffered pitches.
    pub fn recognize_chord(&self, key_center: Option<u8>) -> Option<ChordSymbol> {
        let pitches = self.unique_pitches();
        if pitches.len() < 2 { return None; }

        // Extract unique relative pitches (mod 12)
        let mut rels: Vec<u8> = pitches.iter().map(|p| p % 12).collect();
        rels.sort_unstable();
        rels.dedup();

        // Try each relative pitch as root, match against all known chord types
        let types: Vec<&crate::harmony::ChordType> = TYPES.values().collect();

        for &root_rp in &rels {
            for &ct in &types {
                // Check if all required chord degrees are present in the detected notes
                let required: Vec<u8> = ct.degrees.iter()
                    .map(|d| (root_rp + d.pitch()) % 12)
                    .collect();

                let matched = required.iter()
                    .filter(|r| rels.contains(r))
                    .count();

                // Allow missing 5th
                let perfect_fifth = (root_rp + 7) % 12;
                let has_fifth = rels.contains(&perfect_fifth);
                let min_match = if has_fifth { required.len() - 1 } else { required.len() };

                if matched >= min_match && matched >= 2 {
                    // Found a chord match
                    let note = Note::new(root_rp);
                    let root_name = note.rel_pitch_to_string();
                    let chord_name = format!("{}{}", root_name, ct.name);
                    if let Ok(cs) = ChordSymbol::parse(&chord_name) {
                        return Some(cs);
                    }
                }
            }
        }

        // Fallback: just return the most common pitch as major chord
        if let Some(&most_common) = rels.first() {
            let note = Note::new(most_common);
            let root_name = note.rel_pitch_to_string();
            ChordSymbol::parse(&root_name).ok()
        } else {
            None
        }
    }
}

// ══════════════════════════════════════════════════════════
//  Live Audio Pipeline: mic → pitch → chord
// ══════════════════════════════════════════════════════════

/// Configuration for live audio input.
pub struct LiveConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub pitch_min_hz: f32,
    pub pitch_max_hz: f32,
    pub chord_window_secs: f32,
    pub chord_change_interval_secs: f32, // min time between chord updates
}

impl Default for LiveConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 2048,
            pitch_min_hz: 60.0,
            pitch_max_hz: 1000.0,
            chord_window_secs: 2.0,
            chord_change_interval_secs: 0.5,
        }
    }
}

/// The live audio pipeline state.
pub struct LivePipeline {
    pub config: LiveConfig,
    pub pitch_buffer: PitchBuffer,
    pub current_chord: Option<ChordSymbol>,
    pub chord_history: VecDeque<ChordSymbol>,
    pub last_chord_change: f32,
    pub elapsed_secs: f32,
}

impl LivePipeline {
    pub fn new(config: LiveConfig) -> Self {
        let pitch_buffer = PitchBuffer::new(config.chord_window_secs);
        Self {
            pitch_buffer,
            config,
            current_chord: None,
            chord_history: VecDeque::new(),
            last_chord_change: 0.0,
            elapsed_secs: 0.0,
        }
    }

    /// Process an audio buffer from the microphone.
    /// Returns Some(chord) when a new chord is recognized.
    pub fn process_audio(&mut self, samples: &[f32]) -> Option<ChordSymbol> {
        let dt = samples.len() as f32 / self.config.sample_rate as f32;
        self.elapsed_secs += dt;

        // Detect pitch
        if let Some(freq) = detect_pitch(samples, self.config.sample_rate) {
            let midi = freq_to_midi(freq);
            self.pitch_buffer.push(midi, self.elapsed_secs);

            // Try chord recognition at interval
            if self.elapsed_secs - self.last_chord_change >= self.config.chord_change_interval_secs {
                if let Some(chord) = self.pitch_buffer.recognize_chord(None) {
                    // Only report if chord changed
                    let is_new = self.current_chord.as_ref().map_or(true, |c| c.name != chord.name);
                    if is_new {
                        self.current_chord = Some(chord.clone());
                        self.chord_history.push_back(chord.clone());
                        self.last_chord_change = self.elapsed_secs;
                        return Some(chord);
                    }
                }
            }
        }
        None
    }

    /// Get all unique chords detected so far.
    pub fn chord_progression(&self) -> Vec<ChordSymbol> {
        let mut chords: Vec<ChordSymbol> = self.chord_history.iter().cloned().collect();
        chords.dedup_by(|a, b| a.name == b.name);
        chords
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freq_to_midi() {
        assert_eq!(freq_to_midi(440.0), 69); // A4
        assert_eq!(freq_to_midi(261.63), 60); // C4
    }

    #[test]
    fn test_silence_no_pitch() {
        let silence = vec![0.0f32; 2048];
        assert!(detect_pitch(&silence, 44100).is_none());
    }

    #[test]
    fn test_sine_wave_pitch() {
        let sr = 44100;
        let freq = 440.0; // A4
        let samples: Vec<f32> = (0..2048).map(|i| {
            (2.0 * std::f32::consts::PI * freq * i as f32 / sr as f32).sin()
        }).collect();
        let detected = detect_pitch(&samples, sr);
        assert!(detected.is_some(), "Should detect 440Hz");
        if let Some(f) = detected {
            assert!((f - 440.0).abs() < 5.0, "Got {}Hz, expected ~440Hz", f);
        }
    }

    #[test]
    fn test_pitch_buffer() {
        let mut pb = PitchBuffer::new(0.3);
        pb.push(60, 0.0); // C
        pb.push(64, 0.1); // E
        pb.push(67, 0.2); // G
        let pitches = pb.unique_pitches();
        assert_eq!(pitches.len(), 3);
        assert!(pitches.contains(&60));
        assert!(pitches.contains(&64));
        assert!(pitches.contains(&67));
    }

    #[test]
    fn test_chord_recognition_c_major() {
        let mut pb = PitchBuffer::new(1.0);
        pb.push(60, 0.0); // C4
        pb.push(64, 0.1); // E4
        pb.push(67, 0.1); // G4
        let chord = pb.recognize_chord(None);
        assert!(chord.is_some(), "Should recognize C major");
        assert!(chord.unwrap().name.starts_with('C'));
    }

    #[test]
    fn test_live_pipeline() {
        let config = LiveConfig { chord_change_interval_secs: 0.0, ..Default::default() };
        let mut pipeline = LivePipeline::new(config);

        // Push C major notes directly (simulating what process_audio does after pitch detection)
        pipeline.pitch_buffer.push(60, 0.0); // C4
        pipeline.pitch_buffer.push(64, 0.0); // E4
        pipeline.pitch_buffer.push(67, 0.0); // G4

        if let Some(chord) = pipeline.pitch_buffer.recognize_chord(None) {
            pipeline.current_chord = Some(chord.clone());
            pipeline.chord_history.push_back(chord);
        }

        let progression = pipeline.chord_progression();
        assert!(!progression.is_empty(), "Should detect at least one chord");
        assert!(progression[0].name.starts_with('C'), "Should detect C major, got {}", progression[0].name);
    }
}
