use std::collections::BTreeSet;
use crate::harmony::note::Note;

/// An ordered collection of notes sorted by pitch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Chord {
    notes: BTreeSet<NoteKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
struct NoteKey(u8);

impl From<&Note> for NoteKey { fn from(n: &Note) -> Self { NoteKey(n.pitch) } }

impl Default for Chord {
    fn default() -> Self {
        Self::new()
    }
}

impl Chord {
    pub fn new() -> Self { Self { notes: BTreeSet::new() } }

    pub fn add(&mut self, note: Note) { self.notes.insert(NoteKey::from(&note)); }

    pub fn notes(&self) -> Vec<Note> {
        self.notes.iter().map(|k| Note::new(k.0)).collect()
    }

    pub fn size(&self) -> usize { self.notes.len() }

    pub fn to_absolute_note_string(&self) -> String {
        let ns: Vec<String> = self.notes().iter().map(|n| n.piano_octave_string()).collect();
        format!("[{}]", ns.join(","))
    }
}
