use midly::{Smf, TrackEventKind, MetaMessage};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_beat: f32,
    pub duration_beats: f32,
    pub channel: u8,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedStylePart {
    pub name: String,
    pub size_beats: f32,
    pub channels: HashMap<u8, Vec<ParsedNote>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedStyle {
    pub parts: Vec<ParsedStylePart>,
    pub total_beats: f32,
}

/// Parse a Yamaha style file (.prs/.sty/.yjz = Standard MIDI File with markers).
pub fn parse_style_file<P: AsRef<Path>>(path: P) -> Result<ParsedStyle, String> {
    let data = std::fs::read(path.as_ref())
        .map_err(|e| format!("Cannot read file: {}", e))?;
    let smf = Smf::parse(&data)
        .map_err(|e| format!("Invalid MIDI/SMF file: {}", e))?;

    let mut markers: Vec<(String, f32)> = Vec::new();
    // (ch, pitch, velocity, start_beat, duration_beats)
    let mut all_notes: Vec<(u8, u8, u8, f32, f32)> = Vec::new();

    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int() as f32,
        midly::Timing::Timecode(fps, sf) => fps.as_f32() * sf as f32 * 4.0,
    };

    for track in &smf.tracks {
        let mut abs_tick: u64 = 0;
        // (channel, pitch, velocity, start_tick)
        let mut active: Vec<(u8, u8, u8, u64)> = Vec::new();

        for event in track {
            abs_tick += event.delta.as_int() as u64;

            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    use midly::MidiMessage;
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let v = vel.as_int();
                            if v > 0 {
                                active.push((channel.as_int(), key.as_int(), v, abs_tick));
                            } else {
                                // vel=0 means NoteOff
                                if let Some(pos) = active.iter().position(
                                    |(c, k, _, _)| *c == channel.as_int() && *k == key.as_int()
                                ) {
                                    let (ch, key, vel_val, start) = active.remove(pos);
                                    let dur = (abs_tick - start) as f32 / ticks_per_beat;
                                    let pos_beat = start as f32 / ticks_per_beat;
                                    all_notes.push((ch, key, vel_val, pos_beat, dur));
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            if let Some(pos) = active.iter().position(
                                |(c, k, _, _)| *c == channel.as_int() && *k == key.as_int()
                            ) {
                                let (ch, key, vel_val, start) = active.remove(pos);
                                let dur = (abs_tick - start) as f32 / ticks_per_beat;
                                let pos_beat = start as f32 / ticks_per_beat;
                                all_notes.push((ch, key, vel_val, pos_beat, dur));
                            }
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Meta(meta) => {
                    if let MetaMessage::Marker(marker) = meta {
                        let s = String::from_utf8_lossy(marker).to_string();
                        if !s.is_empty() && !s.starts_with("SFF") && !s.starts_with("SInt") {
                            markers.push((s, abs_tick as f32 / ticks_per_beat));
                        }
                    }
                }
                _ => {}
            }
        }

        // Remaining active notes
        for (ch, key, vel_val, start) in &active {
            let dur = (abs_tick - *start) as f32 / ticks_per_beat;
            let pos_beat = *start as f32 / ticks_per_beat;
            all_notes.push((*ch, *key, *vel_val, pos_beat, dur));
        }
    }

    if markers.is_empty() {
        return Err("No style markers found in file".to_string());
    }

    let total_beats = markers.iter().map(|(_, pos)| *pos).fold(0.0f32, f32::max);

    let mut parts = Vec::new();
    for i in 0..markers.len() {
        let (ref name, start_beat) = markers[i];
        let end_beat = if i + 1 < markers.len() { markers[i + 1].1 } else { total_beats + 4.0 };
        let size = end_beat - start_beat;

        let mut channels: HashMap<u8, Vec<ParsedNote>> = HashMap::new();
        for &(ch, pitch, vel, pos, dur) in &all_notes {
            if pos >= start_beat && pos < end_beat {
                let rel_pos = pos - start_beat;
                channels.entry(ch).or_default().push(ParsedNote {
                    pitch,
                    velocity: vel.max(1), // at least 1, not 0
                    start_beat: rel_pos,
                    duration_beats: dur.min(size - rel_pos).max(0.01),
                    channel: ch,
                });
            }
        }

        parts.push(ParsedStylePart { name: name.clone(), size_beats: size, channels });
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
