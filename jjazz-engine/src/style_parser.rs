use midly::{Smf, TrackEventKind, MetaMessage};
use std::collections::HashMap;
use std::path::Path;

/// A parsed note from a style file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_beat: f32,
    pub duration_beats: f32,
    pub channel: u8,
}

/// A style part: a named section with notes per channel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedStylePart {
    pub name: String,
    pub size_beats: f32,
    /// Notes grouped by channel
    pub channels: HashMap<u8, Vec<ParsedNote>>,
}

/// Parsed style file with markers and notes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedStyle {
    pub parts: Vec<ParsedStylePart>,
    pub total_beats: f32,
}

/// Parse a Yamaha style file (.prs/.sty which is SMF Format 0).
pub fn parse_style_file<P: AsRef<Path>>(path: P) -> Result<ParsedStyle, String> {
    let data = std::fs::read(path.as_ref())
        .map_err(|e| format!("Cannot read file: {}", e))?;
    let smf = Smf::parse(&data)
        .map_err(|e| format!("Invalid MIDI/SMF file: {}", e))?;

    // Step 1: Find all markers and their beat positions
    let mut markers: Vec<(String, f32)> = Vec::new();
    let mut all_notes: Vec<(u8, u8, u8, f32, f32)> = Vec::new(); // (ch, pitch, vel, start_tick, dur_ticks)

    // SMF can be format 0 (single track) or format 1 (multiple)
    // Yamaha styles are usually format 0
    
    // First determine ticks per beat
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int() as f32,
        midly::Timing::Timecode(fps, subframe) => {
            // Timecode: convert to approximate ticks per beat
            fps.as_f32() * subframe as f32 * 4.0
        }
    };

    // Collect markers and notes from all tracks
    for track in &smf.tracks {
        let mut abs_tick: u64 = 0;
        let mut active_notes: Vec<(u8, u8, u8, u64)> = Vec::new(); // (ch, pitch, velocity, start_tick)

        for event in track {
            abs_tick += event.delta.as_int() as u64;

            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    use midly::MidiMessage;
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                active_notes.push((channel.as_int(), key.as_int(), vel.as_int(), abs_tick));
                            } else {
                                // NoteOff (vel=0)
                                active_notes.retain(|(ch, k, _, start)| {
                                    if *ch == channel.as_int() && *k == key.as_int() {
                                        let dur = (abs_tick - *start) as f32 / ticks_per_beat;
                                        let pos = *start as f32 / ticks_per_beat;
                                        all_notes.push((channel.as_int(), key.as_int(), 0, pos, dur));
                                        false
                                    } else { true }
                                });
                            }
                        }
                        MidiMessage::NoteOff { key, vel: _ } => {
                            active_notes.retain(|(ch, k, _, start)| {
                                if *ch == channel.as_int() && *k == key.as_int() {
                                    let dur = (abs_tick - *start) as f32 / ticks_per_beat;
                                    let pos = *start as f32 / ticks_per_beat;
                                    all_notes.push((channel.as_int(), key.as_int(), 0, pos, dur));
                                    false
                                } else { true }
                            });
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Meta(meta) => {
                    if let MetaMessage::Marker(marker) = meta {
                        let marker_str = String::from_utf8_lossy(marker).to_string();
                        if !marker_str.is_empty() && !marker_str.starts_with("SFF") && !marker_str.starts_with("SInt") {
                            let beat_pos = abs_tick as f32 / ticks_per_beat;
                            markers.push((marker_str, beat_pos));
                        }
                    }
                }
                _ => {}
            }
        }

        // Clean up any remaining active notes
        for (ch, key, _vel, start) in &active_notes {
            let dur = (abs_tick - *start) as f32 / ticks_per_beat;
            let pos = *start as f32 / ticks_per_beat;
            all_notes.push((*ch, *key, 0, pos, dur));
        }
    }

    if markers.is_empty() {
        return Err("No style markers found in file".to_string());
    }

    // Step 2: Build style parts from markers
    let mut parts = Vec::new();
    let total_beats = markers.iter().map(|(_, pos)| *pos).fold(0.0f32, f32::max);

    for i in 0..markers.len() {
        let (ref name, start_beat) = markers[i];
        let end_beat = if i + 1 < markers.len() { markers[i + 1].1 } else { total_beats + 4.0 };
        let size = end_beat - start_beat;

        // Collect notes in this beat range
        let mut channels: HashMap<u8, Vec<ParsedNote>> = HashMap::new();
        for &(ch, pitch, _vel, pos, dur) in &all_notes {
            if pos >= start_beat && pos < end_beat {
                let rel_pos = pos - start_beat;
                // Try to get velocity from active_notes
                let velocity: u8 = 100; // default
                channels.entry(ch).or_default().push(ParsedNote {
                    pitch,
                    velocity,
                    start_beat: rel_pos,
                    duration_beats: dur.min(size - rel_pos).max(0.01),
                    channel: ch,
                });
            }
        }

        parts.push(ParsedStylePart {
            name: name.clone(),
            size_beats: size,
            channels,
        });
    }

    Ok(ParsedStyle { parts, total_beats })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_nonexistent() {
        assert!(parse_style_file("nonexistent.sty").is_err());
    }
}
