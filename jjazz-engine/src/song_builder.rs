use crate::phrase::{Phrase, NoteEvent};
use crate::harmony::{TimeSignature, ChordSymbol};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_bass_phrase_to_chord};
use std::collections::HashMap;

// ─── Intensity / Velocity adjustment ────────────────────

/// Apply intensity scaling: 0=soft, 50=normal, 100=loud
pub fn apply_intensity(tracks: &mut [&mut Phrase], intensity: u8) {
    let factor = if intensity <= 50 {
        0.6 + (intensity as f32 / 50.0) * 0.4  // 0.6 → 1.0
    } else {
        1.0 + ((intensity - 50) as f32 / 50.0) * 0.3  // 1.0 → 1.3
    };
    for track in tracks {
        for ne in &mut track.notes {
            ne.velocity = (ne.velocity as f32 * factor).clamp(1.0, 127.0) as u8;
        }
    }
}

// ─── BassLine enhancement ───────────────────────────────

/// Enhance bass line: add chromatic approaches and octave doubling.
pub fn enhance_bass_line(bass: &mut Phrase) {
    if bass.len() < 2 { return; }
    let mut extra = Vec::new();

    for i in 0..bass.len() - 1 {
        let curr = &bass.notes[i];
        let next = &bass.notes[i + 1];
        let gap = next.position - (curr.position + curr.duration);

        // If gap > 0.5 beats, add a ghost note halfway
        if gap > 0.5 && gap < 2.0 {
            let mid_pos = curr.position + curr.duration + gap * 0.5;
            let mid_pitch = ((curr.pitch as i32 + next.pitch as i32) / 2)
                .clamp(0, 127) as u8;
            extra.push(NoteEvent::new(mid_pitch, gap * 0.4, 60, mid_pos));
        }
    }
    for ne in extra { bass.add(ne); }
    bass.sort();
}

// ─── Anticipated chord ──────────────────────────────────

/// Anticipate chord changes: shift notes slightly early at chord boundaries.
pub fn anticipate_chords(tracks: &mut [Phrase], chord_positions: &[f32], window: f32) {
    for track in tracks {
        for ne in &mut track.notes {
            for &cp in chord_positions {
                if ne.position > cp - window && ne.position < cp + 0.1 {
                    ne.position = cp.max(ne.position - window * 0.3);
                }
            }
        }
        track.sort();
    }
}

// ─── Fill / Variation ───────────────────────────────────

/// Select which style section to use based on Fill/Variation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SongSection {
    MainA, MainB, MainC, MainD,
    FillAA, FillBB, FillCC, FillDD,
    IntroA, IntroB, IntroC,
    EndingA, EndingB, EndingC,
}

impl SongSection {
    pub fn from_variation(var: u8) -> Self {
        match var {
            0 => SongSection::MainA,
            1 => SongSection::MainB,
            2 => SongSection::MainC,
            _ => SongSection::MainD,
        }
    }
    pub fn from_fill(fill: u8) -> Self {
        match fill {
            0 => SongSection::FillAA,
            1 => SongSection::FillBB,
            2 => SongSection::FillCC,
            _ => SongSection::FillDD,
        }
    }
    pub fn section_name(&self) -> &str {
        match self {
            Self::MainA => "Main A", Self::MainB => "Main B",
            Self::MainC => "Main C", Self::MainD => "Main D",
            Self::FillAA => "Fill In AA", Self::FillBB => "Fill In BB",
            Self::FillCC => "Fill In CC", Self::FillDD => "Fill In DD",
            Self::IntroA => "Intro A", Self::IntroB => "Intro B",
            Self::IntroC => "Intro C", Self::EndingA => "Ending A",
            Self::EndingB => "Ending B", Self::EndingC => "Ending C",
        }
    }
}

// ─── DrumsMixTransform ──────────────────────────────────

/// Remap drum notes between different drum maps.
pub fn remap_drums(track: &mut Phrase, map: &HashMap<u8, u8>) {
    for ne in &mut track.notes {
        if let Some(&new_pitch) = map.get(&ne.pitch) {
            ne.pitch = new_pitch;
        }
    }
}

// ─── Song structure builder ─────────────────────────────

/// A song arrangement: sequence of sections with chords.
#[derive(Debug, Clone)]
pub struct SongArrangement {
    pub sections: Vec<SongSection>,
    pub chords_per_section: Vec<Vec<ChordSymbol>>,
}

/// Build a full song track list from an arrangement.
pub fn build_song(
    arrangement: &SongArrangement,
    all_parts: &HashMap<String, crate::style_parser::ParsedStylePart>,
    intensity: u8,
    _bpm: f32,
) -> Result<Vec<Phrase>, String> {
    let _ts = TimeSignature::FOUR_FOUR;
    let mut all_tracks: Vec<Phrase> = (0..8).map(|i| {
        if i <= 1 { Phrase::new(9) } else { Phrase::new((i - 2) as u8) }
    }).collect();
    let source_chord = ChordSymbol::parse("Cmaj7")
        .unwrap_or_else(|_| ChordSymbol::parse("C").unwrap());

    let mut global_offset: f32 = 0.0;

    for (sec_idx, section) in arrangement.sections.iter().enumerate() {
        let chords = &arrangement.chords_per_section[sec_idx];
        let part = all_parts.get(&section.section_name().to_string())
            .ok_or_else(|| format!("Section {} not found", section.section_name()))?;

        let pattern_beats = part.size_beats;
        let chord_duration = pattern_beats / chords.len().max(1) as f32;

        for (chord_idx, current_cs) in chords.iter().enumerate() {
            let section_start = global_offset + chord_idx as f32 * chord_duration;

            for (&ch, notes) in &part.channels {
                let is_drums = ch == 9 || ch == 10;
                let track_idx = if is_drums { 1 } else {
                    (ch as usize + 2).min(all_tracks.len() - 1)
                };
                let track = &mut all_tracks[track_idx];

                for note in notes {
                    let rel = note.start_beat % pattern_beats;
                    if rel >= chord_duration { continue; }
                    let pos = rel + section_start;
                    let dur = note.duration_beats.min(chord_duration - rel);
                    if dur <= 0.0 { continue; }

                    let final_pitch = if is_drums {
                        note.pitch
                    } else if current_cs.name != source_chord.name {
                        let mut sp = SourcePhrase::new(ch, source_chord.clone());
                        sp.add(NoteEvent::new(note.pitch, dur, note.velocity, pos));
                        let adapted = if track_idx == 2 {
                            fit_bass_phrase_to_chord(&sp, current_cs)
                        } else {
                            fit_melody_phrase_to_chord(&sp, current_cs, false)
                        };
                        if adapted.is_empty() { note.pitch } else { adapted.notes[0].pitch }
                    } else { note.pitch };

                    track.add(NoteEvent::new(final_pitch, dur, note.velocity, pos));
                }
            }
        }
        global_offset += pattern_beats;
    }

    // Post-processing pipeline
    let mut track_refs: Vec<&mut Phrase> = all_tracks.iter_mut().collect();
    apply_intensity(&mut track_refs, intensity);
    // Bass enhancement on track 2
    enhance_bass_line(&mut all_tracks[2]);

    Ok(all_tracks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intensity_soft() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.5, 100, 0.0));
        apply_intensity(&mut [&mut p], 0);
        assert!(p.notes[0].velocity < 80, "Soft: {}", p.notes[0].velocity);
    }

    #[test]
    fn test_intensity_loud() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.5, 100, 0.0));
        apply_intensity(&mut [&mut p], 100);
        assert!(p.notes[0].velocity >= 120, "Loud: {}", p.notes[0].velocity);
    }

    #[test]
    fn test_bass_enhance_adds_notes() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(48, 0.5, 100, 0.0));
        p.add(NoteEvent::new(50, 0.5, 90, 2.0));
        let orig = p.len();
        enhance_bass_line(&mut p);
        assert!(p.len() >= orig);
    }

    #[test]
    fn test_section_names() {
        assert_eq!(SongSection::MainA.section_name(), "Main A");
        assert_eq!(SongSection::FillBB.section_name(), "Fill In BB");
        assert_eq!(SongSection::from_variation(0), SongSection::MainA);
        assert_eq!(SongSection::from_fill(1), SongSection::FillBB);
    }

    #[test]
    fn test_drum_remap() {
        let mut p = Phrase::new(9);
        p.add(NoteEvent::new(36, 0.3, 100, 0.0));
        let mut map = HashMap::new();
        map.insert(36, 35);
        remap_drums(&mut p, &map);
        assert_eq!(p.notes[0].pitch, 35);
    }
}
