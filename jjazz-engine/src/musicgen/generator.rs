use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};
use crate::style::{AccType, Style};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_chord_phrase_to_chord, fit_bass_phrase_to_chord};
use crate::retrigger::adapt_note;
use crate::patterns::*;

const BASE_OCTAVE: i32 = 24;

pub fn generate_with_style(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, true)
}

pub fn generate_clean(chords: &[ChordSymbol], style: &Style) -> Vec<Phrase> {
    generate_impl(chords, style, false)
}

fn generate_impl(chords: &[ChordSymbol], style: &Style, humanize: bool) -> Vec<Phrase> {
    if chords.is_empty() { return Vec::new(); }
    let ts = TimeSignature::FOUR_FOUR;
    let voice_types = [
        AccType::SubRhythm, AccType::Rhythm, AccType::Bass,
        AccType::Chord1, AccType::Chord2, AccType::Pad,
        AccType::Phrase1, AccType::Phrase2,
    ];
    let mut tracks: Vec<Phrase> = voice_types.iter().map(|at| Phrase::new(at.channel())).collect();
    let ref_chord = &chords[0];
    let part = &style.parts[0];
    let cfg = &part.channel_settings;

    for (bar, current_cs) in chords.iter().enumerate() {
        let bar_start = bar as f32 * ts.nb_natural_beats();
        let prev_cs = if bar > 0 { &chords[bar - 1] } else { current_cs };
        let root_rp = current_cs.root_note.relative_pitch() as i32;
        let root = (BASE_OCTAVE + root_rp).max(0).min(127) as u8;
        let next_root_rp = if bar + 1 < chords.len() {
            chords[bar + 1].root_note.relative_pitch() as i32
        } else { root_rp };
        let next_root = (BASE_OCTAVE + next_root_rp).max(0).min(127) as u8;
        let is_last = bar + 1 >= chords.len();
        let is_bossa = style.name.contains("Bossa");

        for (vi, at) in voice_types.iter().enumerate() {
            let track = &mut tracks[vi];
            let ch_cfg = &cfg[vi];

            let notes: Vec<NoteEvent> = match at {
                AccType::SubRhythm => Vec::new(),
                AccType::Rhythm => if is_bossa { drum_bossa(bar_start) } else { drum_swing(bar_start) },
                AccType::Bass => if is_bossa { bass_bossa(root, bar_start) } else { bass_walking(root, next_root, bar_start, is_last) },
                AccType::Chord1 => comping_bossa_guitar(root, current_cs, bar_start),
                AccType::Chord2 => comping_piano(root, current_cs, bar_start),
                AccType::Pad => pad_sustained(root, current_cs, bar_start, ts.nb_natural_beats()),
                AccType::Phrase1 | AccType::Phrase2 => melody_call_response(root, current_cs, bar_start, bar),
            };

            for ne in notes {
                let final_pitch = if current_cs.name != ref_chord.name && !at.is_drums() {
                    let mut sp = SourcePhrase::new(track.channel, ref_chord.clone());
                    sp.add(ne.clone());
                    let adapted = match at {
                        AccType::Bass => fit_bass_phrase_to_chord(&sp, current_cs),
                        AccType::Chord1 | AccType::Chord2 | AccType::Pad =>
                            fit_chord_phrase_to_chord(&sp, current_cs),
                        _ => fit_melody_phrase_to_chord(&sp, current_cs, false),
                    };
                    if adapted.is_empty() { ne.pitch } else { adapted.notes[0].pitch }
                } else if bar > 0 && ne.position == bar_start && !at.is_drums() {
                    adapt_note(ne.pitch, prev_cs, current_cs, ch_cfg.retrigger_rule).unwrap_or(ne.pitch)
                } else {
                    ne.pitch
                };
                track.add(NoteEvent::new(final_pitch, ne.duration, ne.velocity, ne.position));
            }
        }
    }

    if humanize {
        let hum = Humanizer::new(HumanizerConfig::default());
        for track in &mut tracks { hum.humanize(track); quantize_phrase(track, &ts); }
    } else {
        for track in &mut tracks { quantize_phrase(track, &ts); }
    }
    tracks
}

fn quantize_phrase(p: &mut Phrase, ts: &TimeSignature) {
    let new_notes: Vec<NoteEvent> = p.notes.iter().map(|ne| {
        let pos = Position::from_absolute_beat(ne.position, ts);
        let q = quantize(Quantization::Beat, &pos, ts, 1.0, 99);
        ne.set_position(q.to_absolute_beat(ts))
    }).collect();
    p.notes = new_notes;
    p.sort();
}
