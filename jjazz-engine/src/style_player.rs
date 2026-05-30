use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::style_parser::{parse_style_file, ParsedStylePart};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_chord_phrase_to_chord, fit_bass_phrase_to_chord};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};

/// Generate backing tracks from a Yamaha style file.
/// `bars_per_chord`: how many bars each chord lasts. 0 = full pattern length.
pub fn generate_from_style_file(
    style_path: &str,
    chords: &[ChordSymbol],
    bars_per_chord: u32,
) -> Result<Vec<Phrase>, String> {
    let parsed = parse_style_file(style_path)?;
    let part = parsed.parts.first()
        .ok_or("Style file has no parts".to_string())?;
    generate_from_parsed_part(part, chords, bars_per_chord)
}

pub fn generate_from_parsed_part(
    part: &ParsedStylePart,
    chords: &[ChordSymbol],
    bars_per_chord: u32,
) -> Result<Vec<Phrase>, String> {
    let ts = TimeSignature::FOUR_FOUR;
    let pattern_beats = part.size_beats; // e.g. 32 beats
    let beats_per_chord = if bars_per_chord > 0 {
        bars_per_chord as f32 * ts.nb_natural_beats()
    } else {
        pattern_beats
    };

    let channel_map: &[(u8, usize, bool)] = &[
        (9,  1, true), (10, 1, true),
        (0,  2, false), (1, 3, false), (2, 4, false),
        (3,  5, false), (4, 6, false), (5, 7, false),
    ];

    let mut tracks: Vec<Phrase> = (0..8).map(|i| {
        if i <= 1 { Phrase::new(9) } else { Phrase::new(i as u8 - 2) }
    }).collect();

    let source_chord = ChordSymbol::parse("Cmaj7")
        .unwrap_or_else(|_| ChordSymbol::parse("C").unwrap());

    for (chord_idx, current_cs) in chords.iter().enumerate() {
        let section_start = chord_idx as f32 * beats_per_chord;

        for &(ch, track_idx, is_drums) in channel_map {
            if let Some(notes) = part.channels.get(&ch) {
                let track = &mut tracks[track_idx];

                for note in notes {
                    // Only include notes that fall within the chord's duration
                    let rel_beat = note.start_beat % pattern_beats;
                    if rel_beat >= beats_per_chord { continue; }

                    let pos = rel_beat + section_start;
                    let dur = note.duration_beats.min(beats_per_chord - rel_beat);
                    if dur <= 0.0 { continue; }

                    let final_pitch = if is_drums {
                        note.pitch
                    } else if current_cs.name != source_chord.name {
                        let mut sp = SourcePhrase::new(ch, source_chord.clone());
                        sp.add(NoteEvent::new(note.pitch, dur, note.velocity, pos));
                        let adapted = match track_idx {
                            2 => fit_bass_phrase_to_chord(&sp, current_cs),
                            3 | 4 | 5 => fit_chord_phrase_to_chord(&sp, current_cs),
                            _ => fit_melody_phrase_to_chord(&sp, current_cs, false),
                        };
                        if adapted.is_empty() { note.pitch } else { adapted.notes[0].pitch }
                    } else {
                        note.pitch
                    };

                    track.add(NoteEvent::new(final_pitch, dur, note.velocity, pos));
                }
            }
        }
    }

    let hum = Humanizer::new(HumanizerConfig::default());
    for track in &mut tracks {
        hum.humanize(track);
        let new_notes: Vec<NoteEvent> = track.notes.iter().map(|ne| {
            let pos = Position::from_absolute_beat(ne.position, &ts);
            let q = quantize(Quantization::Beat, &pos, &ts, 1.0, 99);
            ne.set_position(q.to_absolute_beat(&ts))
        }).collect();
        track.notes = new_notes;
        track.sort();
    }
    Ok(tracks)
}
