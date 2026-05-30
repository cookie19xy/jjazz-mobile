use jjazz_engine::live::{LivePipeline, LiveConfig, detect_pitch, freq_to_midi, midi_to_name};

fn main() {
    println!("JJazz Live - Microphone Chord Detection");
    println!("=======================================");

    // Generate a test C major chord and run through the pipeline
    let mut pipeline = LivePipeline::new(LiveConfig { chord_change_interval_secs: 0.0, ..Default::default() });
    let sr = 44100;

    // Simulate C major: C4+E4+G4
    let samples: Vec<f32> = (0..4096).map(|i| {
        let t = i as f32 / sr as f32;
        0.5 * (2.0 * std::f32::consts::PI * 261.63 * t).sin()
            + 0.3 * (2.0 * std::f32::consts::PI * 329.63 * t).sin()
            + 0.3 * (2.0 * std::f32::consts::PI * 392.0 * t).sin()
    }).collect();

    for _ in 0..5 {
        if let Some(chord) = pipeline.process_audio(&samples) {
            println!("Detected: {}", chord.name);
        }
    }

    println!("\nChord progression: {:?}", pipeline.chord_progression().iter().map(|c| &c.name).collect::<Vec<_>>());

    println!("\nPipeline is working. Full mic input needs cpal event loop.");
    println!("But you can integrate it like:");
    println!("  pipeline.process_audio(&mic_buffer) -> Some(ChordSymbol)");
    println!("  → feed into StreamingEngine for live accompaniment");
}
