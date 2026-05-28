use crate::harmony::note::{Note, Accidental};
use serde::{Serialize, Deserialize};

/// A Note with a position in beats on a timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteEvent {
    pub pitch: u8,
    pub duration: f32,
    pub velocity: u8,
    pub position: f32,
    pub accidental: Accidental,
}

impl NoteEvent {
    pub fn new(pitch: u8, duration: f32, velocity: u8, position: f32) -> Self {
        Self { pitch, duration, velocity, position, accidental: Accidental::Flat }
    }

    pub fn from_note(note: &Note, position: f32) -> Self {
        Self { pitch: note.pitch, duration: note.duration, velocity: note.velocity, position, accidental: note.accidental }
    }

    pub fn end_position(&self) -> f32 { self.position + self.duration }

    pub fn set_pitch(&self, pitch: u8) -> Self { Self { pitch, ..*self } }
    pub fn set_duration(&self, duration: f32) -> Self { Self { duration, ..*self } }
    pub fn set_velocity(&self, velocity: u8) -> Self { Self { velocity, ..*self } }
    pub fn set_position(&self, position: f32) -> Self { Self { position, ..*self } }

    pub fn piano_octave_string(&self) -> String {
        Note::with_all(self.pitch, self.duration, self.velocity, self.accidental).piano_octave_string()
    }
}
