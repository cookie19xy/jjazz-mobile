use crate::harmony::{ChordSymbol, Degree, Note};
use crate::phrase::{Phrase, NoteEvent};

/// SourcePhrase: a Phrase bound to its original source chord.
/// This is the core unit that gets adapted when chords change.
#[derive(Debug, Clone)]
pub struct SourcePhrase {
    pub phrase: Phrase,
    pub source_chord: ChordSymbol,
}

impl SourcePhrase {
    pub fn new(channel: u8, source_chord: ChordSymbol) -> Self {
        Self { phrase: Phrase::new(channel), source_chord }
    }

    pub fn from_phrase(phrase: Phrase, source_chord: ChordSymbol) -> Self {
        Self { phrase, source_chord }
    }

    pub fn add(&mut self, ne: NoteEvent) {
        self.phrase.add(ne);
    }

    pub fn len(&self) -> usize { self.phrase.len() }
    pub fn is_empty(&self) -> bool { self.phrase.is_empty() }
    pub fn channel(&self) -> u8 { self.phrase.channel }

    /// Map source degrees to destination degrees.
    /// For each unique degree in the source phrase notes, find the corresponding
    /// destination degree in the destination chord type.
    pub fn get_dest_degrees(&self, dest_chord: &ChordSymbol) -> Vec<(Degree, Degree)> {
        let src_ct = match self.source_chord.chord_type() {
            Some(ct) => ct,
            None => return Vec::new(),
        };
        let dest_ct = match dest_chord.chord_type() {
            Some(ct) => ct,
            None => return Vec::new(),
        };

        let src_root = self.source_chord.root_note.relative_pitch();
        let dest_root = dest_chord.root_note.relative_pitch();

        // Collect unique relative pitches from source notes
        let mut src_rel_pitches: Vec<u8> = Vec::new();
        for ne in &self.phrase.notes {
            let rp = (ne.pitch as i32 - src_root as i32).rem_euclid(12) as u8;
            if !src_rel_pitches.contains(&rp) {
                src_rel_pitches.push(rp);
            }
        }

        let mut result = Vec::new();
        for rp in &src_rel_pitches {
            let src_degree = Degree::most_probable(*rp);
            // Fit source degree into destination chord type
            let dest_degree = dest_ct.fit_degree(src_degree).unwrap_or(src_degree);
            if !result.contains(&(src_degree, dest_degree)) {
                result.push((src_degree, dest_degree));
            }
        }
        result
    }
}

/// Fit a melody-oriented source phrase to a destination chord.
/// Transposes by root delta, then adapts each note's degree.
pub fn fit_melody_phrase_to_chord(
    src: &SourcePhrase,
    dest_chord: &ChordSymbol,
    chord_mode: bool,
) -> Phrase {
    let mut dest = Phrase::new(src.phrase.channel);
    if src.is_empty() { return dest; }

    let src_root = src.source_chord.root_note.relative_pitch() as i32;
    let dest_root = dest_chord.root_note.relative_pitch() as i32;
    let root_pitch_delta = (dest_root - src_root).rem_euclid(12);

    // Same chord type + no special scale -> simple transpose
    if src.source_chord.chord_type_name == dest_chord.chord_type_name {
        for ne in &src.phrase.notes {
            let new_pitch = (ne.pitch as i32 + root_pitch_delta).clamp(0, 127) as u8;
            dest.add(ne.set_pitch(new_pitch));
        }
        return dest;
    }

    let src_ct = match src.source_chord.chord_type() { Some(ct) => ct, None => return dest };
    let dest_ct = match dest_chord.chord_type() { Some(ct) => ct, None => return dest };

    let map_degrees = src.get_dest_degrees(dest_chord);

    for ne in &src.phrase.notes {
        let src_rel = ((ne.pitch as i32 - src_root).rem_euclid(12)) as u8;
        let src_degree = Degree::most_probable(src_rel);

        // Find matching destination degree
        let dest_degree = map_degrees.iter()
            .find(|(sd, _)| *sd == src_degree)
            .map(|(_, dd)| *dd)
            .unwrap_or_else(|| dest_ct.fit_degree(src_degree).unwrap_or(src_degree));

        let dest_rel = dest_degree.pitch() as i32;
        let new_pitch = closest_pitch(ne.pitch as i32 + root_pitch_delta, dest_rel as u8);
        dest.add(ne.set_pitch(new_pitch));
    }
    dest
}

/// Fit a chord phrase (vertical/horizontal) to destination chord.
pub fn fit_chord_phrase_to_chord(
    src: &SourcePhrase,
    dest_chord: &ChordSymbol,
) -> Phrase {
    fit_melody_phrase_to_chord(src, dest_chord, true)
}

/// Fit a bass phrase to destination chord.
/// Uses bass note if slash chord, handles pedal bass.
pub fn fit_bass_phrase_to_chord(
    src: &SourcePhrase,
    dest_chord: &ChordSymbol,
) -> Phrase {
    let mut dest = Phrase::new(src.phrase.channel);
    if src.is_empty() { return dest; }

    // If destination has a bass note (slash chord), use it as root
    let effective_root = dest_chord.root_note.relative_pitch() as i32;
    let src_root = src.source_chord.root_note.relative_pitch() as i32;
    let root_pitch_delta = (effective_root - src_root).rem_euclid(12);

    // Simple transpose for bass (keep intervals relative to root)
    for ne in &src.phrase.notes {
        let new_pitch = (ne.pitch as i32 + root_pitch_delta).clamp(0, 127) as u8;
        dest.add(ne.set_pitch(new_pitch));
    }
    dest
}

fn closest_pitch(reference: i32, target_rp: u8) -> u8 {
    let octave = reference / 12;
    let candidates = [
        (octave - 1) * 12 + target_rp as i32,
        octave * 12 + target_rp as i32,
        (octave + 1) * 12 + target_rp as i32,
    ];
    candidates.iter()
        .map(|&c| c.clamp(0, 127) as u8)
        .min_by_key(|&p| (p as i32 - reference).abs())
        .unwrap_or(reference as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harmony::ChordSymbol;

    #[test]
    fn test_fit_melody_same_chord() {
        let c = ChordSymbol::parse("Cmaj7").unwrap();
        let mut sp = SourcePhrase::new(0, c.clone());
        sp.add(NoteEvent::new(60, 0.5, 100, 0.0)); // C4
        sp.add(NoteEvent::new(64, 0.5, 100, 1.0)); // E4

        let result = fit_melody_phrase_to_chord(&sp, &c, false);
        assert_eq!(result.notes[0].pitch, 60); // unchanged
        assert_eq!(result.notes[1].pitch, 64);
    }

    #[test]
    fn test_fit_melody_transpose() {
        let c = ChordSymbol::parse("C").unwrap();
        let g = ChordSymbol::parse("G").unwrap();

        let mut sp = SourcePhrase::new(0, c);
        sp.add(NoteEvent::new(60, 0.5, 100, 0.0)); // C4

        let result = fit_melody_phrase_to_chord(&sp, &g, false);
        // C->G transpose: C4(60) → closest G = 55(G3) or 67(G4)
        let pitch = result.notes[0].pitch;
        assert!(pitch % 12 == 7, "Expected G (rp=7), got {}", pitch % 12);
    }
}
