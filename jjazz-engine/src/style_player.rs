use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::style_parser::{parse_style_file, ParsedStylePart};
use crate::source_phrase::{SourcePhrase, fit_melody_phrase_to_chord, fit_chord_phrase_to_chord, fit_bass_phrase_to_chord};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};

/// Generate from style file. section: "Main","Fill","Intro","Ending" or "" for first Main.
pub fn generate_from_style_file(
    style_path: &str,
    chords: &[ChordSymbol],
    bars_per_chord: u32,
    section: &str,
) -> Result<(Vec<Phrase>, String), String> {
    let parsed = parse_style_file(style_path)?;
    let search = if section.is_empty() { "Main" } else { section };
    let part = parsed.parts.iter()
        .find(|p| p.name.to_lowercase().starts_with(&search.to_lowercase()))
        .ok_or_else(|| {
            let names: Vec<_> = parsed.parts.iter().map(|p| p.name.as_str()).collect();
            format!("Section '{}' not found. Available: {:?}", section, names)
        })?;
    Ok((generate_from_parsed_part(part, chords, bars_per_chord)?, part.name.clone()))
}

pub fn generate_from_parsed_part(
    part: &ParsedStylePart,
    chords: &[ChordSymbol],
    bars_per_chord: u32,
) -> Result<Vec<Phrase>, String> {
    let ts = TimeSignature::FOUR_FOUR;
    let pattern_beats = part.size_beats;
    let beats_per_chord = if bars_per_chord > 0 {
        bars_per_chord as f32 * ts.nb_natural_beats()
    } else { pattern_beats };

    // Dynamic channel mapping: drums→track1, all others→separate tracks
    let mut used_channels: Vec<u8> = part.channels.keys().copied().collect();
    used_channels.sort();

    let mut tracks: Vec<Phrase> = Vec::new();
    tracks.push(Phrase::new(9)); // SubDrums
    tracks.push(Phrase::new(9)); // Drums
    for &ch in &used_channels {
        if ch != 9 && ch != 10 { tracks.push(Phrase::new(ch)); }
    }
    // Minimum 16 tracks for all possible channels
    while tracks.len() < 16 {
        tracks.push(Phrase::new(((tracks.len() - 2) % 16) as u8));
    }

    let source_chord = ChordSymbol::parse("Cmaj7")
        .unwrap_or_else(|_| ChordSymbol::parse("C").unwrap());

    for (chord_idx, current_cs) in chords.iter().enumerate() {
        let section_start = chord_idx as f32 * beats_per_chord;

        for (&ch, notes) in &part.channels {
            let is_drums = ch == 9 || ch == 10;
            let track_idx = if is_drums { 1 } else {
                used_channels.iter().position(|&c| c == ch).map(|p| p + 2).unwrap_or(2)
            };
            let track = &mut tracks[track_idx];

            for note in notes {
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
                    let adapted = if track_idx == 2 {
                        fit_bass_phrase_to_chord(&sp, current_cs)
                    } else {
                        fit_melody_phrase_to_chord(&sp, current_cs, false)
                    };
                    if adapted.is_empty() { note.pitch } else { adapted.notes[0].pitch }
                } else {
                    note.pitch
                };

                track.add(NoteEvent::new(final_pitch, dur, note.velocity, pos));
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
