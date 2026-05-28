use crate::harmony::ChordSymbol;
use crate::phrase::{Phrase, NoteEvent};
use crate::humanizer::{Humanizer, HumanizerConfig};
use crate::quantizer::{quantize, Quantization};
use crate::harmony::{TimeSignature, Position};


/// Generate a multi-track backing track from chord symbols.
/// Returns one Phrase per track (0=bass, 1=chords, 2=melody).
pub fn generate_backing(chords: &[ChordSymbol]) -> Vec<Phrase> {
    let ts = TimeSignature::FOUR_FOUR;
    let mut bass = Phrase::new(0);
    let mut comp = Phrase::new(1);
    let mut melody = Phrase::new(2);

    let base_octave: i32 = 24; // C1 for bass

    for (bar, cs) in chords.iter().enumerate() {
        let rp = cs.root_note.relative_pitch() as i32;
        let root = (base_octave + rp).max(0).min(127) as u8;
        let bar_start = bar as f32 * ts.nb_natural_beats();

        // Bass: root on beat 1, fifth on beat 3
        bass.add(NoteEvent::new(root, 1.5, 127, bar_start));
        bass.add(NoteEvent::new(root + 7, 0.5, 110, bar_start + 2.0));

        // Comping: chord tones spread across the bar
        if let Some(ct) = cs.chord_type() {
            for (i, d) in ct.degrees.iter().enumerate() {
                if i >= 4 { break; } // max 4 notes
                let pitch = root + d.pitch();
                comp.add(NoteEvent::new(pitch, 0.8, 110, bar_start + i as f32 * 1.0));
            }
        }

        // Melody: simple arpeggio, lower octave, longer notes
        if let Some(ct) = cs.chord_type() {
            for i in 0..4 {
                let d = ct.degrees[i % ct.degrees.len()];
                let pitch = (root + 12 + d.pitch()).min(127); // one octave up
                melody.add(NoteEvent::new(pitch, 1.0, 110, bar_start + i as f32 * 1.0));
            }
        }
    }

    // Apply humanization to all tracks
    let hum = Humanizer::new(HumanizerConfig::default());
    hum.humanize(&mut bass);
    hum.humanize(&mut comp);
    hum.humanize(&mut melody);

    // Quantize all tracks
    quantize_phrase(&mut bass, &ts);
    quantize_phrase(&mut comp, &ts);
    quantize_phrase(&mut melody, &ts);

    vec![bass, comp, melody]
}

fn quantize_phrase(p: &mut Phrase, ts: &TimeSignature) {
    let mut new_notes: Vec<NoteEvent> = Vec::new();
    for ne in p.notes.iter() {
        let pos = Position::from_absolute_beat(ne.position, ts);
        let q = quantize(Quantization::Beat, &pos, ts, 1.0, 99);
        let new_beat = q.to_absolute_beat(ts);
        new_notes.push(ne.set_position(new_beat));
    }
    p.notes = new_notes;
    p.sort();
}
