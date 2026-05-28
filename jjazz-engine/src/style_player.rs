use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::style_parser::{parse_style_file, ParsedStylePart};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_chord_phrase_to_chord, fit_bass_phrase_to_chord};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};

/// Generate backing tracks directly from a Yamaha style file (.prs/.sty/.yjz).
pub fn generate_from_style_file(
    style_path: &str,
    chords: &[ChordSymbol],
) -> Result<Vec<Phrase>, String> {
    let parsed = parse_style_file(style_path)?;
    let part = parsed.parts.first()
        .ok_or("Style file has no parts".to_string())?;
    generate_from_parsed_part(part, chords)
}

/// Convert a parsed style part's channel-based notes to AccType-based tracks.
pub fn generate_from_parsed_part(
    part: &ParsedStylePart,
    chords: &[ChordSymbol],
) -> Result<Vec<Phrase>, String> {
    let ts = TimeSignature::FOUR_FOUR;
    let beats_per_bar = ts.nb_natural_beats(); // 4.0

    // Map channels to our 8-track layout:
    // ch9,10→drums, ch0→bass, ch1→guitar, ch2→piano, ch3→pad, ch4→brass, ch5→piano2
    let channel_map = [
        (9u8, 1usize),  // Rhythm drums → track 1
        (10u8, 1usize), // also drums
        (0u8, 2usize),  // Bass
        (1u8, 3usize),  // Guitar (Chord1)
        (2u8, 4usize),  // Piano (Chord2)
        (3u8, 5usize),  // Pad
        (4u8, 6usize),  // Phrase1 (Brass)
        (5u8, 7usize),  // Phrase2 (Piano2)
    ];

    let mut tracks: Vec<Phrase> = (0..8).map(|i| {
        if i == 0 { Phrase::new(9) } // SubRhythm on ch9
        else if i == 1 { Phrase::new(9) } // Rhythm on ch9
        else { Phrase::new(i as u8 - 2) } // Bass=ch0, Guitar=ch1, etc.
    }).collect();

    // Style files use Cmaj7 as source chord
    let source_chord = ChordSymbol::parse("Cmaj7")
        .unwrap_or_else(|_| ChordSymbol::parse("C").unwrap());

    for (bar, current_cs) in chords.iter().enumerate() {
        let bar_start = bar as f32 * beats_per_bar;

        for &(ch, track_idx) in &channel_map {
            if let Some(notes) = part.channels.get(&ch) {
                let is_drums = ch == 9 || ch == 10;
                let track = &mut tracks[track_idx];

                for note in notes {
                    let pos = note.start_beat + bar_start;
                    let dur = note.duration_beats;

                    let final_pitch = if is_drums {
                        note.pitch // drums play as-is
                    } else if current_cs.name != source_chord.name {
                        // Build a mini SourcePhrase for chord adaptation
                        let mut sp = SourcePhrase::new(ch, source_chord.clone());
                        sp.add(NoteEvent::new(note.pitch, dur, note.velocity, pos));
                        let adapted = match track_idx {
                            2 => fit_bass_phrase_to_chord(&sp, current_cs),
                            3 | 4 | 5 => fit_chord_phrase_to_chord(&sp, current_cs),
                            _ => fit_melody_phrase_to_chord(&sp, current_cs, false),
                        };
                        if adapted.is_empty() { note.pitch }
                        else { adapted.notes[0].pitch }
                    } else {
                        note.pitch
                    };

                    track.add(NoteEvent::new(final_pitch, dur, note.velocity, pos));
                }
            }
        }
    }

    // Humanize + quantize
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
