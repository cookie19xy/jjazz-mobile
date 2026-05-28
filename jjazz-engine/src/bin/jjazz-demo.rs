use std::env;
use std::fs::File;
use std::io::Write;
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::musicgen::generate_with_style;
use jjazz_engine::style::Style;
use jjazz_engine::synth::SynthEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: jjazz-demo [soundfont.sf2] <和弦...>");
        eprintln!("示例: jjazz-demo Dm7 G7 Cmaj7");
        eprintln!("     jjazz-demo myfont.sf2 Dm7 G7 Cmaj7");
        return;
    }

    // Auto-detect SoundFont: first arg = .sf2 → use it, otherwise look locally
    let (sf2_path, chord_strs): (String, &[String]) = if args[1].ends_with(".sf2") || args[1].ends_with(".sf3") {
        (args[1].clone(), &args[2..])
    } else {
        let candidates = ["JJazzLab-SoundFont.sf2", "TimGM6mb.sf2"];
        match candidates.iter().find(|f| std::path::Path::new(f).exists()) {
            Some(f) => (f.to_string(), &args[1..]),
            None => {
                eprintln!("找不到 SoundFont 文件！请将 .sf2 放在当前目录。");
                return;
            }
        }
    };

    // 1. Parse chords
    println!("解析和弦...");
    let chords: Vec<ChordSymbol> = chord_strs.iter()
        .map(|s| ChordSymbol::parse(s).expect(&format!("无法解析: {}", s)))
        .collect();
    for cs in &chords {
        println!("  {} -> {}  root={}", cs.name, cs.chord_type_name, cs.root_note);
    }

    // 2. Generate with style
    let style = Style::bossanova();
    println!("生成伴奏 (风格: {})...", style.name);
    let tracks = generate_with_style(&chords, &style);

    let names = ["SubDrums","Drums","Bass","Guitar","Piano","Pad","Brass","Piano2"];
    let mut total_notes = 0;
    for (i, t) in tracks.iter().enumerate() {
        if t.len() > 0 {
            println!("  {}: {} notes", names[i], t.len());
            total_notes += t.len();
        }
    }
    println!("  总计: {} 个音符", total_notes);

    // 3. Render
    println!("加载 SoundFont...");
    let mut synth = SynthEngine::from_file(sf2_path).expect("SoundFont 加载失败");
    println!("渲染音频...");
    let audio = synth.render_tracks(&tracks, 120.0);

    let peak = audio.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let nz = audio.iter().filter(|s| s.abs() > 0.0001).count();
    println!("  完成: {:.1}s, 峰值={:.3}, 非零={}/{}",
        audio.len() as f32 / 88200.0, peak, nz, audio.len());

    // 4. WAV
    let _ = std::fs::create_dir("output");
    let wav_path = "output/output.wav";
    write_wav(wav_path, &audio, 44100);
    let size_mb = std::fs::metadata(wav_path).map(|m| m.len() as f64 / 1_000_000.0).unwrap_or(0.0);
    println!("\n✅ {} ({:.1} MB)", wav_path, size_mb);
}

fn write_wav(path: &str, samples: &[f32], sample_rate: u32) {
    let mut f = File::create(path).unwrap();
    let data_len = samples.len() as u32 * 2;
    let file_len = 44 + data_len;
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(file_len - 8).to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap();
    f.write_all(&sample_rate.to_le_bytes()).unwrap();
    f.write_all(&(sample_rate * 4).to_le_bytes()).unwrap();
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
