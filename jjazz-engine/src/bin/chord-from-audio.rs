use std::fs::File;
use std::io::{Read, Write};
use jjazz_engine::live::{LivePipeline, LiveConfig, detect_pitch, freq_to_midi};
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::style_parser::parse_style_file;
use jjazz_engine::style_player::generate_from_parsed_part;
use jjazz_engine::synth::SynthEngine;

fn read_wav_mono(path: &str) -> Result<(Vec<f32>, u32), String> {
    let mut f = File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
    let mut header = [0u8; 44];
    f.read_exact(&mut header).map_err(|e| format!("Bad header: {}", e))?;

    let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
    let bits = u16::from_le_bytes([header[34], header[35]]);
    let channels = u16::from_le_bytes([header[22], header[23]]);

    let mut raw = Vec::new();
    f.read_to_end(&mut raw).map_err(|e| format!("Read error: {}", e))?;

    let samples: Vec<f32> = match bits {
        16 => {
            let count = raw.len() / 2 / channels as usize;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let idx = i * 2 * channels as usize;
                let sample = i16::from_le_bytes([raw[idx], raw[idx + 1]]);
                out.push(sample as f32 / 32768.0);
            }
            out
        }
        _ => return Err(format!("Unsupported bit depth: {}", bits)),
    };

    Ok((samples, sample_rate))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: chord-from-audio <input.wav> [style.prs]");
        return;
    }

    let wav_path = &args[1];
    let style_path = args.get(2)
        .map(|s| s.clone())
        .unwrap_or_else(|| "styles/Latin/PopBossa2.S631.prs".to_string());

    println!("Reading: {}", wav_path);
    let (samples, sample_rate) = match read_wav_mono(wav_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); return; }
    };
    println!("  {} samples, {} Hz, {:.1}s",
        samples.len(), sample_rate, samples.len() as f32 / sample_rate as f32);

    // Pitch detection + chord recognition
    let chunk_size = 4096usize;
    let chord_interval_secs = 0.3;
    let mut pipeline = LivePipeline::new(LiveConfig {
        sample_rate,
        buffer_size: chunk_size,
        chord_window_secs: 2.0,
        chord_change_interval_secs: chord_interval_secs,
        ..Default::default()
    });

    println!("\nDetecting chords...");
    let total_chunks = samples.len() / chunk_size;
    let mut last_report = 0;

    for ci in 0..total_chunks {
        let start = ci * chunk_size;
        let end = (start + chunk_size).min(samples.len());
        let chunk = &samples[start..end];

        let detected = pipeline.process_audio(chunk);

        let progress = (ci * 100) / total_chunks;
        if let Some(chord) = detected {
            println!("  [{:3}%] {:5.1}s → {}", progress, pipeline.elapsed_secs, chord.name);
            last_report = ci;
        } else if ci - last_report > 100 {
            print!("\r  [{:3}%] {:5.1}s", progress, pipeline.elapsed_secs);
            last_report = ci;
        }
    }

    let chords = pipeline.chord_progression();
    // Filter: only keep clean chord types, deduplicate adjacent
    let mut filtered: Vec<ChordSymbol> = Vec::new();
    for c in &chords {
        let ct = c.chord_type_name.clone();
        // Skip sus/dim/aug variants - keep clean triads and 7ths
        let is_clean = !ct.contains("sus") && !ct.contains("dim") && !ct.contains("aug")
            && !ct.contains("b5") && !ct.contains("alt");
        if is_clean {
            if filtered.last().map_or(true, |prev: &ChordSymbol| prev.name != c.name) {
                filtered.push(c.clone());
            }
        }
    }
    // Also remove duplicates overall
    filtered.dedup_by(|a, b| a.name == b.name);

    println!("\nFiltered to {} clean chords:", filtered.len());
    for c in &filtered {
        println!("  {}", c.name);
    }

    if filtered.is_empty() {
        println!("No clean chords detected! Using default progression.");
        let default: Vec<ChordSymbol> = ["C", "G", "Am", "F"].iter()
            .map(|s| ChordSymbol::parse(s).unwrap()).collect();
        render_accompaniment(&default, &style_path, wav_path);
        return;
    }

    render_accompaniment(&filtered, &style_path, wav_path);
}

fn render_accompaniment(chords: &[ChordSymbol], style_path: &str, input_path: &str) {
    println!("\nGenerating accompaniment with: {}", style_path);

    let parsed = match parse_style_file(style_path) {
        Ok(p) => p,
        Err(e) => { eprintln!("Style error: {}", e); return; }
    };
    let part = match parsed.parts.iter().find(|p| p.name.starts_with("Main")) {
        Some(p) => p,
        None => { eprintln!("No Main section in style"); return; }
    };

    let tracks = match generate_from_parsed_part(part, chords, 2) {
        Ok(t) => t,
        Err(e) => { eprintln!("Generation error: {}", e); return; }
    };

    let sf2 = if std::path::Path::new("JJazzLab-SoundFont.sf2").exists() {
        "JJazzLab-SoundFont.sf2"
    } else {
        eprintln!("No SoundFont found"); return;
    };

    let mut synth = SynthEngine::from_file(sf2).expect("SF2 load failed");
    let audio = synth.render_tracks(&tracks, 120.0);

    // Save to same folder as input
    let out_path = std::path::Path::new(input_path)
        .with_file_name("accompaniment.wav");
    write_wav(out_path.to_str().unwrap(), &audio, 44100);
    println!("\n✅ {}", out_path.display());
}

fn write_wav(path: &str, samples: &[f32], sr: u32) {
    let mut f = File::create(path).unwrap();
    let data_len = samples.len() as u32 * 2;
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(44 + data_len - 8).to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
    f.write_all(&2u16.to_le_bytes()).unwrap(); // stereo
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
