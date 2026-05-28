use jjazz_engine::harmony::*;
use jjazz_engine::phrase::*;
use jjazz_engine::humanizer::*;
use jjazz_engine::quantizer::*;
use jjazz_engine::musicgen::*;

#[test]
fn test_parse_chords() {
    let cs = ChordSymbol::parse("Dm7").unwrap();
    assert_eq!(cs.name, "Dm7");
    assert_eq!(cs.root_note.rel_pitch_to_string(), "D");
    assert_eq!(cs.chord_type_name, "m7");

    let cs = ChordSymbol::parse("G7").unwrap();
    assert_eq!(cs.chord_type_name, "7");

    let cs = ChordSymbol::parse("Cmaj7").unwrap();
    assert_eq!(cs.root_note.rel_pitch_to_string(), "C");
}

#[test]
fn test_chord_type_attributes() {
    let cs = ChordSymbol::parse("Dm7").unwrap();
    let ct = cs.chord_type().unwrap();
    assert!(ct.is_minor());
    assert!(ct.is_seventh());
    assert!(ct.is_seventh_minor());

    let cs = ChordSymbol::parse("G7").unwrap();
    let ct = cs.chord_type().unwrap();
    assert!(ct.is_major());
    assert!(ct.is_seventh_minor());
}

#[test]
fn test_generate_backing() {
    let chords = vec![
        ChordSymbol::parse("Dm7").unwrap(),
        ChordSymbol::parse("G7").unwrap(),
        ChordSymbol::parse("Cmaj7").unwrap(),
    ];

    let tracks = generate_backing(&chords);
    assert_eq!(tracks.len(), 3); // bass, comp, melody

    // Bass track should have 6 notes (3 bars × 2 notes)
    assert_eq!(tracks[0].len(), 6);

    // Comp track should have at least 6 notes
    assert!(tracks[1].len() >= 6);

    // Melody track should have 12 notes (3 bars × 4 notes)
    assert_eq!(tracks[2].len(), 12);

    // All notes should be within reasonable bounds
    for track in &tracks {
        for ne in &track.notes {
            assert!(ne.pitch <= 127);
            assert!(ne.velocity <= 127);
            assert!(ne.position >= 0.0);
            assert!(ne.duration > 0.0);
        }
    }
}

#[test]
fn test_humanizer_changes_notes() {
    let mut phrase = Phrase::new(0);
    for i in 0..20 {
        phrase.add(NoteEvent::new(60 + i as u8, 0.5, 100, i as f32));
    }

    let original_count = phrase.len();
    let hum = Humanizer::new(HumanizerConfig::default());
    hum.humanize(&mut phrase);

    assert_eq!(phrase.len(), original_count);
    // At least some notes should have shifted
    let changed = phrase.notes.iter().any(|ne| {
        (ne.position - ne.position.round()).abs() > 0.001 || ne.velocity != 100
    });
    assert!(changed, "Humanizer should modify some notes");
}

#[test]
fn test_quantizer_snaps_to_beat() {
    let pos = Position::new(1, 2.3);
    let q = quantize(Quantization::Beat, &pos, &TimeSignature::FOUR_FOUR, 1.0, 10);
    assert_eq!(q.bar, 1);
    assert!((q.beat - 2.0).abs() < 0.01, "Should snap to beat 2.0, got {}", q.beat);
}

#[test]
fn test_json_roundtrip() {
    let cs = ChordSymbol::parse("Dm7").unwrap();
    let json = serde_json::to_string(&cs).unwrap();
    let cs2: ChordSymbol = serde_json::from_str(&json).unwrap();
    assert_eq!(cs.name, cs2.name);
    assert_eq!(cs.chord_type_name, cs2.chord_type_name);
}

#[test]
fn test_end_to_end() {
    let input = "Dm7 G7 Cmaj7";
    let chords: Vec<ChordSymbol> = input
        .split_whitespace()
        .map(|s| ChordSymbol::parse(s).unwrap())
        .collect();

    assert_eq!(chords.len(), 3);

    let tracks = generate_backing(&chords);
    assert_eq!(tracks.len(), 3);

    println!("=== Generated backing track ===");
    for (i, track) in tracks.iter().enumerate() {
        let name = ["Bass", "Comping", "Melody"][i];
        println!("--- {} ({} notes) ---", name, track.len());
        for ne in &track.notes {
            println!("  {:>5} pos={:<6.3} dur={:<4.2} vel={}", ne.piano_octave_string(), ne.position, ne.duration, ne.velocity);
        }
    }
}
