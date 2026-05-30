use crate::harmony::{ChordSymbol, Degree, TimeSignature};
use crate::phrase::{Phrase, NoteEvent};

/// Walking bass generator: constructs bass lines from chord progressions.
/// Implements the same algorithm as JJSwing's WalkingPhraseBuilder.
pub struct WalkingBassGenerator {
    pub min_velocity: u8,
    pub max_velocity: u8,
    pub use_chromatic: bool,
    pub use_octave_jumps: bool,
}

impl Default for WalkingBassGenerator {
    fn default() -> Self {
        Self { min_velocity: 70, max_velocity: 115, use_chromatic: true, use_octave_jumps: true }
    }
}

impl WalkingBassGenerator {
    /// Build a walking bass line for a chord sequence.
    /// `beats_per_chord`: quarter notes per chord.
    pub fn build(
        &self,
        chords: &[ChordSymbol],
        beats_per_chord: f32,
        tempo: u16,
    ) -> Phrase {
        let mut phrase = Phrase::new(0);
        if chords.is_empty() { return phrase; }
        let base_octave: i32 = 24;
        let mut pos: f32 = 0.0;
        for (ci, cs) in chords.iter().enumerate() {
            let next_cs = chords.get(ci + 1);
            let root = (base_octave + cs.root_note.relative_pitch() as i32) as u8;

            // Build chord tones
            let chord_tones = self.chord_tones(cs, root);

            // Determine beat pattern based on time signature
            let num_beats = beats_per_chord as u32;
            for beat in 0..num_beats {
                let beat_pos = pos + beat as f32;
                let pitch = match beat % num_beats {
                    // Beat 1: root (strong)
                    b if b == 0 => {
                        let vel = self.max_velocity;
                        let p = root;
                        // Add octave jump for interest
                        if self.use_octave_jumps && ci % 2 == 0 && beat == 0 {
                            let upper = (root as i32 + 12).min(127) as u8;
                            phrase.add(NoteEvent::new(upper, 0.1, vel - 5, beat_pos));
                            phrase.add(NoteEvent::new(p, 0.9, vel, beat_pos + 0.1));
                            continue;
                        }
                        p
                    }
                    // Last beat: chromatic approach to next root
                    b if b == num_beats - 1 && self.use_chromatic => {
                        let target = if let Some(next) = next_cs {
                            let nr = base_octave + next.root_note.relative_pitch() as i32;
                            nr as u8
                        } else {
                            // Return to root an octave up
                            (root as i32 + 12).min(127) as u8
                        };
                        // Approach from half step below or above
                        let approach = if target > root { target - 1 } else { target + 1 };
                        approach.clamp(0, 127)
                    }
                    // Middle beats: chord tones
                    _ => {
                        let idx = beat as usize % chord_tones.len().max(1);
                        let tone = chord_tones[idx];
                        // Add some scale passing tones on beat 3 in 4/4
                        if num_beats == 4 && beat == 2 && chord_tones.len() > 3 {
                            // Use the next chord tone for variety
                            chord_tones[(idx + 1) % chord_tones.len()]
                        } else {
                            tone
                        }
                    }
                };

                let vel = if beat == 0 { self.max_velocity }
                    else if beat == num_beats - 1 { self.min_velocity + 10 }
                    else { self.min_velocity + 15 };

                let dur = if beat == num_beats - 1 { 0.7 } else { 0.9 };
                phrase.add(NoteEvent::new(pitch, dur, vel, beat_pos));
            }
            pos += beats_per_chord;
        }
        phrase
    }

    /// Get chord tone pitches for a chord symbol.
    fn chord_tones(&self, cs: &ChordSymbol, root: u8) -> Vec<u8> {
        let mut tones = Vec::new();

        // Root
        tones.push(root);

        if let Some(ct) = cs.chord_type() {
            // Use preferred degrees: root, 5th, 3rd, 7th
            let degrees = [Degree::Root, Degree::Fifth, Degree::Third, Degree::SeventhFlat, Degree::Seventh];
            for &d in &degrees {
                let pitch = root as i32 + d.pitch() as i32;
                let pitch = pitch.clamp(0, 127) as u8;
                if !tones.contains(&pitch) {
                    // Check if this degree exists in the chord
                    let deg_in_chord = ct.degrees.iter().any(|&cd| cd == d || cd.pitch() == d.pitch());
                    if deg_in_chord || d == Degree::Root || d == Degree::Fifth {
                        tones.push(pitch);
                    }
                }
            }
        }

        // Ensure at least 2 tones
        if tones.len() < 2 {
            tones.push((root as i32 + 7).clamp(0, 127) as u8); // fifth
        }
        if tones.len() < 3 {
            tones.push((root as i32 + 4).clamp(0, 127) as u8); // third
        }

        tones
    }
}

// ═══════════════════════════════════════════════════════════
//  MIDI Export: write standard MIDI file from phrase tracks
// ═══════════════════════════════════════════════════════════

/// Write phrase tracks to a standard MIDI file (SMF Format 0).
pub fn write_midi_file(tracks: &[Phrase], bpm: u16, path: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;

    let ticks_per_quarter: u16 = 480;
    let us_per_quarter = 60_000_000 / bpm as u32;

    // Collect all MIDI events: (tick, channel, message)
    let mut events: Vec<(u32, u8, u8, u8, u8)> = Vec::new(); // (tick, channel, status_byte, data1, data2)

    // Tempo event at tick 0
    events.push((0, 0, 0xFF, 0x51, 0x03));
    // Actually tempo is a meta event, handled separately

    for track in tracks {
        let ch = track.channel;
        for ne in &track.notes {
            let tick_on = (ne.position * ticks_per_quarter as f32) as u32;
            let tick_off = ((ne.position + ne.duration) * ticks_per_quarter as f32) as u32;

            if ch == 9 {
                // Drums: note on/off on channel 9
                events.push((tick_on, 9, 0x99, ne.pitch, ne.velocity));
                events.push((tick_off, 9, 0x89, ne.pitch, 0));
            } else {
                events.push((tick_on, ch, 0x90, ne.pitch, ne.velocity));
                events.push((tick_off, ch, 0x80, ne.pitch, 0));
            }
        }
    }

    events.sort_by_key(|(tick, _, _, _, _)| *tick);

    // Write MIDI header
    f.write_all(b"MThd")?;
    f.write_all(&[0, 0, 0, 6])?;  // header size
    f.write_all(&[0, 0])?;        // format 0
    f.write_all(&[0, 1])?;        // 1 track
    f.write_all(&ticks_per_quarter.to_be_bytes())?;

    // Write track chunk
    let mut track_data = Vec::new();

    // Tempo meta event
    write_var_len(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x51, 0x03]);
    track_data.extend_from_slice(&us_per_quarter.to_be_bytes()[1..4]);

    // Time signature meta event (4/4)
    write_var_len(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x58, 0x04, 4, 2, 24, 8]);

    let mut last_tick: u32 = 0;
    for (tick, ch, status, data1, data2) in &events {
        let delta = tick - last_tick;
        write_var_len(&mut track_data, delta);
        if *status == 0xFF {
            // Meta already handled above, skip
            continue;
        }
        let real_status = if *ch == 9 && *status == 0x90 { 0x99u8 }
            else if *ch == 9 && *status == 0x80 { 0x89u8 }
            else { *status | *ch };
        track_data.push(real_status);
        track_data.push(*data1);
        track_data.push(*data2);
        last_tick = *tick;
    }

    // End of track
    write_var_len(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    f.write_all(b"MTrk")?;
    f.write_all(&(track_data.len() as u32).to_be_bytes())?;
    f.write_all(&track_data)?;

    Ok(())
}

fn write_var_len(buf: &mut Vec<u8>, mut value: u32) {
    let mut bytes = Vec::new();
    bytes.push((value & 0x7F) as u8);
    value >>= 7;
    while value > 0 {
        bytes.push((value & 0x7F | 0x80) as u8);
        value >>= 7;
    }
    bytes.reverse();
    buf.extend_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walking_bass_produces_notes() {
        let gen = WalkingBassGenerator::default();
        let chords = vec![
            ChordSymbol::parse("C").unwrap(),
            ChordSymbol::parse("F").unwrap(),
            ChordSymbol::parse("G").unwrap(),
        ];
        let phrase = gen.build(&chords, 4.0, 120);
        assert!(phrase.len() > 0, "Walking bass should produce notes");
        // Each bar should have at least 4 quarter notes
        assert!(phrase.len() >= 9, "Expected >= 9 notes, got {}", phrase.len());
    }

    #[test]
    fn test_walking_bass_chromatic_approach() {
        let gen = WalkingBassGenerator::default();
        let chords = vec![
            ChordSymbol::parse("C").unwrap(),
            ChordSymbol::parse("F").unwrap(),
        ];
        let phrase = gen.build(&chords, 4.0, 120);
        // The last beat before F should be a chromatic approach
        let last_before_change = phrase.notes.iter()
            .find(|n| n.position >= 3.0 && n.position < 4.0);
        assert!(last_before_change.is_some(), "Should have approach note before chord change");
    }

    #[test]
    fn test_midi_export() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.5, 100, 0.0));
        p.add(NoteEvent::new(64, 0.5, 90, 1.0));

        let path = "output/test_export.mid";
        let _ = std::fs::create_dir("output");
        write_midi_file(&[p], 120, path).unwrap();
        assert!(std::path::Path::new(path).exists());
        let size = std::fs::metadata(path).unwrap().len();
        assert!(size > 50, "MIDI file too small: {} bytes", size);
    }
}
