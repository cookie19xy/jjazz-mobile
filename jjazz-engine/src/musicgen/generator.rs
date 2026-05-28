use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};
use crate::style::{AccType, Style};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_chord_phrase_to_chord, fit_bass_phrase_to_chord};
use crate::retrigger::adapt_note;

const BASE_OCTAVE: i32 = 24;

pub fn generate_with_style(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, true)
}

pub fn generate_clean(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, false)
}

fn generate_impl(chords: &[ChordSymbol], style: &Style, humanize: bool) -> Vec<Phrase> {
    if chords.is_empty() { return Vec::new(); }
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

    // Use first chord as reference for source phrase construction
    let ref_chord = &chords[0];

    for (bar, current_cs) in chords.iter().enumerate() {
        let bar_start = bar as f32 * ts.nb_natural_beats();
        let prev_cs = if bar > 0 { &chords[bar - 1] } else { current_cs };
        let root_rp = current_cs.root_note.relative_pitch() as i32;
        let root = (BASE_OCTAVE + root_rp).max(0).min(127) as u8;
        let cfg = &part.channel_settings;

        for (vi, at) in voice_types.iter().enumerate() {
            let track = &mut tracks[vi];
            let ch_cfg = &cfg[vi];

            // Build a SourcePhrase anchored to the FIRST chord for pattern, 
            // then adapt to current chord
            let src_root = ref_chord.root_note.relative_pitch() as i32;
            let ref_root = (BASE_OCTAVE + src_root).max(0).min(127) as u8;

            let notes: Vec<NoteEvent> = match at {
                AccType::SubRhythm => Vec::new(),
                AccType::Rhythm => {
                    make_drum_pattern(bar_start)
                }
                AccType::Bass => {
                    make_bass_pattern(ref_root, ref_chord, bar_start, ref_chord)
                }
                AccType::Chord1 => {
                    make_chord_pattern(ref_root, ref_chord, bar_start, true, ref_chord)
                }
                AccType::Chord2 => {
                    make_chord_pattern(ref_root, ref_chord, bar_start, false, ref_chord)
                }
                AccType::Pad => {
                    make_pad_pattern(ref_root, ref_chord, bar_start, ts, ref_chord)
                }
                AccType::Phrase1 | AccType::Phrase2 => {
                    make_melody_pattern(ref_root, ref_chord, bar_start, ref_chord)
                }
            };

            for ne in notes {
                // Apply chord adaptation if different from reference chord
                let final_pitch = if current_cs.name != ref_chord.name {
                    // Build a mini source phrase to adapt this single note
                    let mut sp = SourcePhrase::new(track.channel, ref_chord.clone());
                    sp.add(ne.clone());
                    let adapted = match at {
                        AccType::Bass => fit_bass_phrase_to_chord(&sp, current_cs),
                        AccType::Chord1 | AccType::Chord2 | AccType::Pad => {
                            fit_chord_phrase_to_chord(&sp, current_cs)
                        }
                        _ => fit_melody_phrase_to_chord(&sp, current_cs, false),
                    };
                    if adapted.is_empty() { ne.pitch } 
                    else { adapted.notes[0].pitch }
                } else {
                    // Also apply retrigger on chord change boundaries
                    if bar > 0 && ne.position == bar_start {
                        adapt_note(ne.pitch, prev_cs, current_cs, ch_cfg.retrigger_rule)
                            .unwrap_or(ne.pitch)
                    } else {
                        ne.pitch
                    }
                };

                track.add(NoteEvent::new(final_pitch, ne.duration, ne.velocity, ne.position));
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

fn make_drum_pattern(bar_start: f32) -> Vec<NoteEvent> {
    let mut notes = Vec::new();
    notes.push(NoteEvent::new(36, 0.3, 110, bar_start));       // Kick 1
    notes.push(NoteEvent::new(36, 0.3, 100, bar_start + 2.0)); // Kick 3
    notes.push(NoteEvent::new(38, 0.2, 100, bar_start + 1.0)); // Snare 2
    notes.push(NoteEvent::new(38, 0.2, 100, bar_start + 3.0)); // Snare 4
    for i in 0..8 {
        notes.push(NoteEvent::new(42, 0.1, 70, bar_start + i as f32 * 0.5)); // HH
    }
    notes
}

fn make_bass_pattern(root: u8, _chord: &ChordSymbol, bar_start: f32, _ref_chord: &ChordSymbol) -> Vec<NoteEvent> {
    let pattern = [(0.0, 1.5), (2.0, 0.5), (2.5, 0.5), (3.0, 0.5)];
    pattern.iter().map(|&(offset, dur)| {
        let pitch = if offset == 0.0 { root }
        else if offset < 3.0 { root + 7 }
        else { root + 12 };
        NoteEvent::new(pitch, dur, 110, bar_start + offset)
    }).collect()
}

fn make_chord_pattern(root: u8, chord: &ChordSymbol, bar_start: f32, is_chord1: bool, _ref_chord: &ChordSymbol) -> Vec<NoteEvent> {
    let pattern: Vec<(f32, f32)> = if is_chord1 {
        vec![(0.0, 0.6), (1.0, 0.4), (2.0, 0.5), (3.0, 0.4)]
    } else {
        vec![(0.0, 0.8), (2.0, 0.6), (3.0, 0.5)]
    };
    if let Some(ct) = chord.chord_type() {
        return pattern.iter().enumerate().map(|(pi, &(offset, dur))| {
            let deg = ct.degrees[pi % ct.degrees.len()];
            let pitch = (root + 12 + deg.pitch()).min(127);
            NoteEvent::new(pitch, dur, 95, bar_start + offset)
        }).collect();
    }
    Vec::new()
}

fn make_pad_pattern(root: u8, chord: &ChordSymbol, bar_start: f32, ts: TimeSignature, _ref_chord: &ChordSymbol) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        return ct.degrees.iter().take(3).map(|&d| {
            let pitch = (root + 12 + d.pitch()).min(127);
            NoteEvent::new(pitch, ts.nb_natural_beats() - 0.05, 75, bar_start)
        }).collect();
    }
    Vec::new()
}

fn make_melody_pattern(root: u8, chord: &ChordSymbol, bar_start: f32, _ref_chord: &ChordSymbol) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        return (0..4).map(|i| {
            let d = ct.degrees[i % ct.degrees.len()];
            let pitch = (root + 12 + d.pitch()).min(127);
            NoteEvent::new(pitch, 0.5, 100, bar_start + i as f32 * 1.0)
        }).collect();
    }
    Vec::new()
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
