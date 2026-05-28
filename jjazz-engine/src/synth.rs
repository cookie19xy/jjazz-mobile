use rustysynth::{Synthesizer, SynthesizerSettings, SoundFont};
use crate::phrase::Phrase;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// SoundFont-based audio renderer. Renders Phrase → stereo audio samples.
pub struct SynthEngine {
    synthesizer: Synthesizer,
}

impl SynthEngine {
    /// Create from a .sf2 file path.
    pub fn from_file<P: AsRef<Path>>(sf2_path: P) -> Result<Self, String> {
        let mut file = File::open(sf2_path.as_ref())
            .map_err(|e| format!("Cannot open SoundFont: {}", e))?;
        let sf2 = Arc::new(SoundFont::new(&mut file)
            .map_err(|e| format!("Failed to parse SoundFont: {}", e))?);

        let mut settings = SynthesizerSettings::new(44100);
        settings.enable_reverb_and_chorus = false; // keep clean, no effect noise
        let synth = Synthesizer::new(&sf2, &settings)
            .map_err(|e| format!("Failed to create synth: {}", e))?;

        Ok(Self { synthesizer: synth })
    }

    pub fn sample_rate(&self) -> i32 { 44100 }

    pub fn program_change(&mut self, channel: u8, program: u8) {
        self.synthesizer.process_midi_message(channel as i32, 0xC0, program as i32, 0);
    }

    pub fn reset(&mut self) {
        self.synthesizer.reset();
        // Silence any hanging voices after reset
        self.synthesizer.note_off_all(true);
    }

    /// Render multiple Phrase tracks to stereo interleaved f32 samples.
    pub fn render_tracks(&mut self, tracks: &[Phrase], bpm: f32) -> Vec<f32> {
        let sr = self.sample_rate() as f32;
        let mut events: Vec<(usize, u8, u8, u8, bool)> = Vec::new();
        let mut total_beats = 0.0f32;

        for track in tracks {
            for ne in &track.notes {
                let on_sample = (ne.position * 60.0 / bpm * sr) as usize;
                let off_sample = ((ne.position + ne.duration) * 60.0 / bpm * sr) as usize;
                events.push((on_sample, track.channel, ne.pitch, ne.velocity, true));
                events.push((off_sample, track.channel, ne.pitch, 0, false));
                total_beats = total_beats.max(ne.position + ne.duration);
            }
        }
        events.sort_by_key(|e| e.0);

        let total_samples = ((total_beats + 1.0) * 60.0 / bpm * sr) as usize;
        self.reset();

        // Standard GM instruments
        // Set instruments for all 8 AccTypes
        // ch0: bass(33), ch1: guitar(26), ch2: piano(1), ch3: strings(50), 
        // ch4: brass(62), ch5: piano2(2), ch9: drums(0)
        self.program_change(0, 33);
        self.program_change(1, 26);
        self.program_change(2, 1);
        self.program_change(3, 50);
        self.program_change(4, 62);
        self.program_change(5, 2);
        // ch9 is drums, no program change needed

        // Default master volume (no artificial boost)
        self.synthesizer.set_master_volume(1.0);

        let mut output = vec![0.0f32; total_samples * 2];
        let mut event_idx = 0;

        for sample in 0..total_samples {
            while event_idx < events.len() && events[event_idx].0 == sample {
                let (_, ch, key, vel, is_on) = events[event_idx];
                if is_on {
                    self.synthesizer.note_on(ch as i32, key as i32, vel as i32);
                } else {
                    self.synthesizer.note_off(ch as i32, key as i32);
                }
                event_idx += 1;
            }
            let mut l = [0.0f32; 1];
            let mut r = [0.0f32; 1];
            self.synthesizer.render(&mut l[..], &mut r[..]);
            output[sample * 2] = l[0];
            output[sample * 2 + 1] = r[0];
        }

        // Fade out last 0.1s to avoid click
        let fade_samples = (0.1 * sr) as usize;
        if output.len() > fade_samples * 2 {
            let start = output.len() - fade_samples * 2;
            for i in 0..fade_samples {
                let gain = 1.0 - (i as f32 / fade_samples as f32);
                output[start + i * 2] *= gain;
                output[start + i * 2 + 1] *= gain;
            }
        }

        output
    }
}
