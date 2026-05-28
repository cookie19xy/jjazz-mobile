use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};
use crate::style::{AccType, Style};
use crate::retrigger::adapt_note;

/// Generate backing tracks using style-based multi-track arrangement (with humanization).
pub fn generate_with_style(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, true)
}

/// Generate without humanization (deterministic, for golden tests).
pub fn generate_clean(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, false)
}

fn generate_impl(chords: &[ChordSymbol], style: &Style, humanize: bool) -> Vec<Phrase> {
    let part = &style.parts[0];
    let ts = TimeSignature::FOUR_FOUR;
    let voice_types = [
        AccType::SubRhythm, AccType::Rhythm, AccType::Bass,
        AccType::Chord1, AccType::Chord2, AccType::Pad,
        AccType::Phrase1, AccType::Phrase2,
    ];
    let mut tracks: Vec<Phrase> = voice_types.iter()
        .map(|at| Phrase::new(at.channel()))
        .collect();

    for (bar, current_cs) in chords.iter().enumerate() {
        let bar_start = bar as f32 * ts.nb_natural_beats();
        let prev_cs = if bar > 0 { &chords[bar - 1] } else { current_cs };
        let root_rp = current_cs.root_note.relative_pitch() as i32;
        let base_octave: i32 = 24;
        let root = (base_octave + root_rp).max(0).min(127) as u8;

        for (vi, at) in voice_types.iter().enumerate() {
            let cfg = &part.channel_settings[vi];
            let track = &mut tracks[vi];
            match at {
                AccType::SubRhythm | AccType::Rhythm => {
                    if *at == AccType::Rhythm {
                        track.add(NoteEvent::new(36, 0.3, 110, bar_start));
                        track.add(NoteEvent::new(36, 0.3, 100, bar_start + 2.0));
                        track.add(NoteEvent::new(38, 0.2, 100, bar_start + 1.0));
                        track.add(NoteEvent::new(38, 0.2, 100, bar_start + 3.0));
                        for i in 0..8 {
                            track.add(NoteEvent::new(42, 0.1, 70, bar_start + i as f32 * 0.5));
                        }
                    }
                }
                AccType::Bass => {
                    let pattern = [(0.0, 1.5), (2.0, 0.5), (2.5, 0.5), (3.0, 0.5)];
                    for &(offset, dur) in &pattern {
                        let pitch = if offset == 0.0 { root }
                        else if offset < 3.0 { root + 7 }
                        else {
                            let next_rp = if bar + 1 < chords.len() {
                                chords[bar + 1].root_note.relative_pitch()
                            } else { current_cs.root_note.relative_pitch() };
                            let next_root = (base_octave + next_rp as i32).max(0).min(127) as u8;
                            if next_root > root { next_root - 1 } else { next_root + 1 }
                        };
                        let final_pitch = if bar > 0 && offset == 0.0 {
                            adapt_note(pitch, prev_cs, current_cs, cfg.retrigger_rule).unwrap_or(pitch)
                        } else { pitch };
                        track.add(NoteEvent::new(final_pitch, dur, 110, bar_start + offset));
                    }
                }
                AccType::Chord1 | AccType::Chord2 => {
                    let pattern: Vec<(f32, f32)> = match *at {
                        AccType::Chord1 => vec![(0.0, 0.6), (1.0, 0.4), (2.0, 0.5), (3.0, 0.4)],
                        _ => vec![(0.0, 0.8), (2.0, 0.6), (3.0, 0.5)],
                    };
                    if let Some(ct) = current_cs.chord_type() {
                        for (pi, &(offset, dur)) in pattern.iter().enumerate() {
                            let deg = ct.degrees[pi % ct.degrees.len()];
                            let pitch = (root + 12 + deg.pitch()).min(127);
                            let final_pitch = if bar > 0 {
                                adapt_note(pitch, prev_cs, current_cs, cfg.retrigger_rule).unwrap_or(pitch)
                            } else { pitch };
                            track.add(NoteEvent::new(final_pitch, dur, 95, bar_start + offset));
                        }
                    }
                }
                AccType::Pad => {
                    if let Some(ct) = current_cs.chord_type() {
                        for &d in ct.degrees.iter().take(3) {
                            let pitch = (root + 12 + d.pitch()).min(127);
                            let final_pitch = if bar > 0 {
                                adapt_note(pitch, prev_cs, current_cs, cfg.retrigger_rule).unwrap_or(pitch)
                            } else { pitch };
                            track.add(NoteEvent::new(final_pitch, ts.nb_natural_beats() - 0.05, 75, bar_start));
                        }
                    }
                }
                AccType::Phrase1 | AccType::Phrase2 => {
                    if let Some(ct) = current_cs.chord_type() {
                        for i in 0..4 {
                            let d = ct.degrees[i % ct.degrees.len()];
                            let pitch = (root + 12 + d.pitch()).min(127);
                            track.add(NoteEvent::new(pitch, 0.5, 100, bar_start + i as f32 * 1.0));
                        }
                    }
                }
            }
        }
    }

    if humanize {
        let hum = Humanizer::new(HumanizerConfig::default());
        for track in &mut tracks {
            hum.humanize(track);
            quantize_phrase(track, &ts);
        }
    } else {
        for track in &mut tracks {
            quantize_phrase(track, &ts);
        }
    }
    tracks
}

fn quantize_phrase(p: &mut Phrase, ts: &TimeSignature) {
    let mut new_notes = Vec::new();
    for ne in &p.notes {
        let pos = Position::from_absolute_beat(ne.position, ts);
        let q = quantize(Quantization::Beat, &pos, ts, 1.0, 99);
        let new_beat = q.to_absolute_beat(ts);
        new_notes.push(ne.set_position(new_beat));
    }
    p.notes = new_notes;
    p.sort();
}
