use std::collections::HashMap;
use crate::phrase::note_event::NoteEvent;

/// A collection of NoteEvents sorted by position.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Phrase {
    pub channel: u8,
    pub notes: Vec<NoteEvent>,
}

impl Phrase {
    pub fn new(channel: u8) -> Self { Self { channel, notes: Vec::new() } }

    pub fn add(&mut self, ne: NoteEvent) {
        let idx = self.notes.binary_search_by(|n| n.position.partial_cmp(&ne.position).unwrap().then_with(|| n.pitch.cmp(&ne.pitch)));
        match idx { Ok(i) | Err(i) => self.notes.insert(i, ne) }
    }

    pub fn remove(&mut self, ne: &NoteEvent) {
        self.notes.retain(|n| n.position != ne.position || n.pitch != ne.pitch);
    }

    pub fn replace_all(&mut self, map: &HashMap<NoteEvent, NoteEvent>) {
        for (old, new) in map {
            self.notes.retain(|n| n.position != old.position || n.pitch != old.pitch);
            self.add(new.clone());
        }
        self.sort();
    }

    pub fn len(&self) -> usize { self.notes.len() }
    pub fn is_empty(&self) -> bool { self.notes.is_empty() }
    pub fn sort(&mut self) { self.notes.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap()); }

    pub fn iter(&self) -> impl Iterator<Item = &NoteEvent> { self.notes.iter() }

    pub fn beat_range(&self) -> (f32, f32) {
        if self.notes.is_empty() { return (0.0, 0.0); }
        let min = self.notes.iter().map(|n| n.position).fold(f32::MAX, f32::min);
        let max = self.notes.iter().map(|n| n.end_position()).fold(0.0, f32::max);
        (min, max)
    }
}
