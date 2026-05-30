use std::env;
use std::fs;
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::musicgen::generate_clean;
use jjazz_engine::style::Style;
use serde::Serialize;

#[derive(Serialize)]
struct TrackJson {
    channel: u8,
    notes: Vec<NoteJson>,
}

#[derive(Serialize)]
struct NoteJson {
    pitch: u8,
    position: f32,
    duration: f32,
    velocity: u8,
    note_name: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: jjazz-export <和弦...>");
        return;
    }

    let chords: Vec<ChordSymbol> = args[1..].iter()
        .map(|s| ChordSymbol::parse(s).unwrap_or_else(|_| panic!("无法解析: {}", s)))
        .collect();

    let style = Style::bossanova();
    let tracks = generate_clean(&chords, &style);

    let _names = ["SubDrums","Drums","Bass","Guitar","Piano","Pad","Brass","Piano2"];
    let mut output = Vec::new();

    for (_i, track) in tracks.iter().enumerate() {
        let mut notes = Vec::new();
        for ne in &track.notes {
            notes.push(NoteJson {
                pitch: ne.pitch,
                position: ne.position,
                duration: ne.duration,
                velocity: ne.velocity,
                note_name: ne.piano_octave_string(),
            });
        }
        notes.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        output.push(TrackJson { channel: track.channel, notes });
    }

    let json = serde_json::to_string_pretty(&output).unwrap();
    let name = chords.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join("_");
    let path = format!("golden/{}.json", name);
    fs::create_dir_all("golden").ok();
    fs::write(&path, &json).unwrap();
    println!("{} ({} tracks, {} notes)", path, tracks.len(), tracks.iter().map(|t| t.len()).sum::<usize>());
}
