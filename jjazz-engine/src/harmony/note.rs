use std::fmt;

/// Musical note with pitch (0-127), duration in beats, velocity (0-127).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Note {
    pub pitch: u8,
    pub duration: f32,
    pub velocity: u8,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Accidental { Flat, Sharp }

impl Default for Accidental { fn default() -> Self { Accidental::Flat } }

pub const PITCH_MIN: u8 = 0;
pub const PITCH_STD: u8 = 60;
pub const PITCH_MAX: u8 = 127;
pub const OCTAVE_STD: i32 = 4;
pub const VELOCITY_STD: u8 = 100;

pub const NOTES_FLAT: [&str; 12] = ["C","Db","D","Eb","E","F","Gb","G","Ab","A","Bb","B"];
pub const NOTES_SHARP: [&str; 12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];

impl Note {
    pub fn new(pitch: u8) -> Self {
        assert!(pitch <= PITCH_MAX);
        Self { pitch, duration: 1.0, velocity: VELOCITY_STD, accidental: Accidental::Flat }
    }

    pub fn with_dur_vel(pitch: u8, duration: f32, velocity: u8) -> Self {
        assert!(pitch <= PITCH_MAX && velocity <= 127 && duration > 0.0);
        Self { pitch, duration, velocity, accidental: Accidental::Flat }
    }

    pub fn with_all(pitch: u8, duration: f32, velocity: u8, accidental: Accidental) -> Self {
        assert!(pitch <= PITCH_MAX && velocity <= 127 && duration > 0.0);
        Self { pitch, duration, velocity, accidental }
    }

    pub fn relative_pitch(&self) -> u8 { self.pitch % 12 }

    pub fn octave(&self) -> i32 { self.pitch as i32 / 12 }

    pub fn rel_pitch_to_string(&self) -> String {
        match self.accidental {
            Accidental::Flat => NOTES_FLAT[self.relative_pitch() as usize].to_string(),
            Accidental::Sharp => NOTES_SHARP[self.relative_pitch() as usize].to_string(),
        }
    }

    pub fn piano_octave_string(&self) -> String {
        format!("{}{}", self.rel_pitch_to_string(), self.octave() - 1)
    }

    /// Transpose by `semitones` semitones.
    pub fn transpose(&self, semitones: i32) -> Self {
        let new_pitch = (self.pitch as i32 + semitones).clamp(0, 127) as u8;
        Self { pitch: new_pitch, ..*self }
    }

    /// Get the closest pitch matching `rel_pitch` to this note.
    pub fn closest_pitch(&self, rel_pitch: u8) -> u8 {
        let up = self.upper_pitch(rel_pitch, true);
        let low = self.lower_pitch(rel_pitch, true);
        if up - self.pitch > self.pitch - low { low } else { up }
    }

    pub fn lower_pitch(&self, rel_pitch: u8, inclusive: bool) -> u8 {
        let base = self.octave() * 12 + rel_pitch as i32;
        let mut p = base;
        let rp = self.relative_pitch();
        if (rp == rel_pitch && !inclusive) || rel_pitch > rp {
            p = (self.octave() - 1) * 12 + rel_pitch as i32;
        }
        p.max(0) as u8
    }

    pub fn upper_pitch(&self, rel_pitch: u8, inclusive: bool) -> u8 {
        let base = self.octave() * 12 + rel_pitch as i32;
        let mut p = base;
        let rp = self.relative_pitch();
        if (rp == rel_pitch && !inclusive) || rel_pitch < rp {
            p = (self.octave() + 1) * 12 + rel_pitch as i32;
        }
        p.min(127) as u8
    }

    pub fn equals_relative_pitch(&self, other: &Note) -> bool {
        self.relative_pitch() == other.relative_pitch()
    }
}

impl Default for Note {
    fn default() -> Self { Self::new(PITCH_STD) }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.piano_octave_string())
    }
}

/// Parse a note from a string like "C", "Eb", "F#", "C!4", "Bb!3"
pub fn parse_note(s: &str) -> Result<Note, String> {
    let s = s.trim();
    if s.is_empty() { return Err("empty".into()); }

    let mut accidental = Accidental::Flat;
    let (degree_str, rest) = if s.len() >= 2 && (s.as_bytes()[1] == b'b' || s.as_bytes()[1] == b'#') {
        (&s[..2], &s[2..])
    } else {
        (&s[..1], &s[1..])
    };

    let mut rel_pitch: i32 = -1;
    if degree_str.eq_ignore_ascii_case("Cb") { rel_pitch = 11; accidental = Accidental::Flat; }
    else if degree_str.eq_ignore_ascii_case("B#") { rel_pitch = 0; accidental = Accidental::Sharp; }
    else if degree_str.eq_ignore_ascii_case("E#") { rel_pitch = 5; accidental = Accidental::Sharp; }
    else if degree_str.eq_ignore_ascii_case("Fb") { rel_pitch = 4; accidental = Accidental::Flat; }
    else {
        for i in 0..12 {
            if degree_str.eq_ignore_ascii_case(NOTES_FLAT[i]) { rel_pitch = i as i32; accidental = Accidental::Flat; break; }
            if degree_str.eq_ignore_ascii_case(NOTES_SHARP[i]) { rel_pitch = i as i32; accidental = Accidental::Sharp; break; }
        }
    }
    if rel_pitch < 0 { return Err(format!("invalid note: {}", s)); }

    let octave = if let Some(idx) = rest.find('!') {
        rest[idx+1..].parse::<i32>().map_err(|e| format!("bad octave: {}", e))?
    } else {
        OCTAVE_STD
    };

    let pitch = (octave * 12 + rel_pitch) as u8;
    if pitch > 127 { return Err(format!("pitch out of range: {}", pitch)); }

    Ok(Note { pitch, duration: 1.0, velocity: VELOCITY_STD, accidental })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_pitch() {
        let c4 = Note::new(60);
        assert_eq!(c4.pitch, 60);
        assert_eq!(c4.relative_pitch(), 0);
        assert_eq!(c4.octave(), 5);
        assert_eq!(c4.piano_octave_string(), "C4");
    }

    #[test]
    fn test_transpose() {
        let c4 = Note::new(60);
        assert_eq!(c4.transpose(2).pitch, 62);
    }

    #[test]
    fn test_closest_pitch() {
        let c4 = Note::new(60);
        assert_eq!(c4.closest_pitch(11), 59); // B3
        assert_eq!(c4.closest_pitch(1), 61);  // Db4
    }

    #[test]
    fn test_parse() {
        let n = parse_note("Eb!3").unwrap();
        assert_eq!(n.pitch, 39); // 3*12+3
        let n2 = parse_note("F#").unwrap();
        assert_eq!(n2.pitch, 54); // 4*12+6
    }
}
