use std::env;
use std::fs::File;
use std::io::Write;
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::musicgen::generate_backing;
use jjazz_engine::synth::SynthEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: jjazz-demo <soundfont.sf2> <和弦...>");
        eprintln!("示例: jjazz-demo TimGM6mb.sf2 Dm7 G7 Cmaj7");
        return;
    }

    let sf2_path = &args[1];
    let chord_strs = &args[2..];

    // 1. Parse chords
    println!("解析和弦...");
    let chords: Vec<ChordSymbol> = chord_strs.iter()
        .map(|s| ChordSymbol::parse(s).expect(&format!("无法解析和弦: {}", s)))
        .collect();
    for cs in &chords {
        println!("  {} -> {}  root={}", cs.name, cs.chord_type_name, cs.root_note);
    }

    // 2. Generate backing track
    println!("生成伴奏...");
    let tracks = generate_backing(&chords);
    for (i, t) in tracks.iter().enumerate() {
        let name = ["Bass", "Comp", "Melody"][i];
        println!("  {}: {} notes", name, t.len());
    }

    // 3. Load SoundFont + render
    println!("加载 SoundFont: {} ...", sf2_path);
    let mut synth = SynthEngine::from_file(sf2_path)
        .expect("无法加载 SoundFont");

    println!("渲染音频...");
    let audio = synth.render_tracks(&tracks, 120.0);
    println!("  渲染完成: {} samples ({:.1}s)", audio.len() / 2, audio.len() as f32 / 88200.0);

    // 4. Write WAV file
    let wav_path = "output.wav";
    write_wav(wav_path, &audio, 44100);
    println!("\n✅ 输出: {} ({:.1} MB)", wav_path, std::fs::metadata(wav_path).map(|m| m.len() as f64 / 1_000_000.0).unwrap_or(0.0));
    println!("用任意音频播放器打开 output.wav 即可听到效果！");
}

fn write_wav(path: &str, samples: &[f32], sample_rate: u32) {
    let mut f = File::create(path).unwrap();
    let data_len = samples.len() as u32 * 2; // 16-bit = 2 bytes per sample
    let file_len = 44 + data_len;

    // WAV header
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(file_len - 8).to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();

    // fmt chunk
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
    f.write_all(&1u16.to_le_bytes()).unwrap();  // PCM
    f.write_all(&2u16.to_le_bytes()).unwrap();  // stereo
    f.write_all(&sample_rate.to_le_bytes()).unwrap();
    f.write_all(&(sample_rate * 4).to_le_bytes()).unwrap(); // byte rate
    f.write_all(&4u16.to_le_bytes()).unwrap();  // block align
    f.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample

    // data chunk
    f.write_all(b"data").unwrap();
    f.write_all(&data_len.to_le_bytes()).unwrap();

    // Convert f32 to i16
    for s in samples {
        let clamped = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        f.write_all(&clamped.to_le_bytes()).unwrap();
    }
}
