use std::env;
use std::fs::File;
use std::io::Write;
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::style_player::generate_from_style_file;
use jjazz_engine::synth::SynthEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: play-style <style> [-b bars] [-p part] <和弦...>");
        eprintln!("示例: play-style style.yjz Dm7 G7 Cmaj7");
        eprintln!("     play-style style.yjz -b 2 -p 1 Dm7 G7  (每和弦2小节, P2变奏)");
        return;
    }

    let style_path = &args[1];
    let mut bars_per_chord: u32 = 0;
    let mut part_index: usize = 0;
    let mut i = 2;

    while i < args.len() {
        if args[i] == "-b" && i + 1 < args.len() {
            bars_per_chord = args[i + 1].parse().unwrap_or(0);
            i += 2;
        } else if args[i] == "-p" && i + 1 < args.len() {
            part_index = args[i + 1].parse().unwrap_or(0);
            i += 2;
        } else {
            break;
        }
    }

    let chords: Vec<ChordSymbol> = args[i..].iter()
        .map(|s| ChordSymbol::parse(s).expect(&format!("无法解析: {}", s)))
        .collect();

    println!("加载风格: {} (P{}, {}小节/和弦)", style_path, part_index + 1,
        if bars_per_chord > 0 { bars_per_chord.to_string() } else { "full".into() });
    let tracks = generate_from_style_file(style_path, &chords, bars_per_chord, part_index)
        .expect("Failed to load style");

    let names = ["SubDrums","Drums","Bass","Guitar","Piano","Pad","Brass","Piano2"];
    let mut total = 0;
    for (i, t) in tracks.iter().enumerate() {
        if t.len() > 0 {
            println!("  {}: {} notes", names[i], t.len());
            total += t.len();
        }
    }
    println!("  总计: {} notes", total);

    // Auto-detect SoundFont
    let sf2 = if std::path::Path::new("JJazzLab-SoundFont.sf2").exists() {
        "JJazzLab-SoundFont.sf2"
    } else if std::path::Path::new("TimGM6mb.sf2").exists() {
        "TimGM6mb.sf2"
    } else {
        eprintln!("No SoundFont found in current directory");
        return;
    };

    println!("渲染音频...");
    let mut synth = SynthEngine::from_file(sf2).expect("SoundFont load failed");
    let audio = synth.render_tracks(&tracks, 120.0);

    let _ = std::fs::create_dir("output");
    let wav_path = "output/style_output.wav";
    write_wav(wav_path, &audio, 44100);
    let size = std::fs::metadata(wav_path).map(|m| m.len() as f64 / 1e6).unwrap_or(0.0);
    println!("\n✅ {} ({:.1} MB)", wav_path, size);
}

fn write_wav(path: &str, samples: &[f32], sr: u32) {
    let mut f = File::create(path).unwrap();
    let data_len = samples.len() as u32 * 2;
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(44 + data_len - 8).to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap();
    f.write_all(&sr.to_le_bytes()).unwrap();
    f.write_all(&(sr * 4).to_le_bytes()).unwrap();
    f.write_all(&4u16.to_le_bytes()).unwrap();
    f.write_all(&16u16.to_le_bytes()).unwrap();
    f.write_all(b"data").unwrap();
    f.write_all(&data_len.to_le_bytes()).unwrap();
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let norm = if peak > 0.001 { 0.95 / peak } else { 1.0 };
    for s in samples {
        let clamped = (s * norm * 32767.0).clamp(-32768.0, 32767.0) as i16;
        f.write_all(&clamped.to_le_bytes()).unwrap();
    }
}
