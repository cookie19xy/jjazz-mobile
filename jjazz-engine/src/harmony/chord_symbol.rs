use crate::harmony::note::{Note, parse_note};
use crate::harmony::chord_type::ChordType;
use crate::harmony::chord_types;
use crate::harmony::chord::Chord;

/// A jazz chord symbol like "Cm7", "F7b9", "Dbmaj7/F".
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChordSymbol {
    pub name: String,
    pub root_note: Note,
    pub bass_note: Note,
    pub chord_type_name: String,
}

impl ChordSymbol {
    /// Default "C" major triad.
    pub fn new() -> Self {
        Self {
            name: "C".into(),
            root_note: Note::new(0),
            bass_note: Note::new(0),
            chord_type_name: "".into(),
        }
    }

    /// Parse a chord symbol string. Examples: "C", "Dm7", "F#7b9", "Bbmaj7/F"
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() { return Err("empty".into()); }

        // Find root note (first 1-2 chars may be note name)
        let root_end = if s.len() >= 2 && (s.as_bytes()[1] == b'b' || s.as_bytes()[1] == b'#') { 2 } else { 1 };

        // Check for octave spec "!n"
        let (root_str, rest_start) = if let Some(idx) = s.find('!') {
            if idx < root_end {
                let end = s[root_end..].find('!').map(|i| root_end + i).unwrap_or(s.len());
                (&s[..end], end)
            } else {
                (&s[..root_end], root_end)
            }
        } else {
            (&s[..root_end], root_end)
        };

        let root = parse_note(root_str)?;

        // Find bass note after '/'
        let (ct_str, bass_str) = if let Some(slash) = s[rest_start..].find('/') {
            let slash_pos = rest_start + slash;
            (&s[rest_start..slash_pos], Some(s[slash_pos+1..].trim()))
        } else {
            (&s[rest_start..], None)
        };

        let ct_str = ct_str.trim();
        let ct_name = find_chord_type_name(ct_str);
        let ct = chord_types::get(&ct_name).ok_or_else(|| format!("unknown chord type: '{}'", ct_str))?;

        let (bass, name) = if let Some(bs) = bass_str {
            let b = parse_note(bs)?;
            (b, format!("{}{}/{}", root.rel_pitch_to_string(), ct.name, b.rel_pitch_to_string()))
        } else {
            (root, format!("{}{}", root.rel_pitch_to_string(), ct.name))
        };

        Ok(Self { name, root_note: root, bass_note: bass, chord_type_name: ct.name.clone() })
    }

    pub fn chord_type(&self) -> Option<&ChordType> {
        chord_types::get(&self.chord_type_name)
    }

    /// Get the chord notes at a given root pitch.
    pub fn get_chord(&self, root_pitch: u8) -> Option<Chord> {
        let ct = self.chord_type()?;
        let c = ct.chord();
        let mut result = Chord::new();
        for n in c.notes() {
            result.add(Note::new(root_pitch + n.pitch));
        }
        Some(result)
    }

    pub fn is_slash_chord(&self) -> bool {
        !self.root_note.equals_relative_pitch(&self.bass_note)
    }
}

/// Map common aliases to canonical chord type names.
fn find_chord_type_name(input: &str) -> String {
    let lower = input.to_lowercase();
    match lower.as_str() {
        "" => "".into(),
        "m" | "min" | "mi" | "-" => "m".into(),
        "maj" | "maj7" | "M7" | "ma7" | "△" | "△7" => "M7".into(),
        "dim" | "°" | "o" => "dim".into(),
        "aug" | "+" => "aug".into(),
        "sus" | "sus4" => "sus".into(),
        "sus2" => "2".into(),
        "7" => "7".into(),
        "m7" | "min7" | "mi7" | "-7" => "m7".into(),
        "dim7" | "°7" => "dim7".into(),
        "m7b5" | "ø" | "halfdim" => "m7b5".into(),
        "mmaj7" | "minmaj7" => "mM7".into(),
        "6" => "6".into(),
        "m6" | "min6" | "-6" => "m6".into(),
        "9" => "9".into(),
        "maj9" | "M9" | "ma9" => "M9".into(),
        "m9" | "min9" | "-9" => "m9".into(),
        "7b9" => "7b9".into(),
        "7#9" => "7#9".into(),
        "7b5" => "7b5".into(),
        "7#5" | "7+5" => "7#5".into(),
        "7#11" => "7#11".into(),
        "7alt" | "alt" => "7alt".into(),
        "13" => "13".into(),
        "7sus" | "7sus4" => "7sus".into(),
        _ => input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_basic() {
        let cs = ChordSymbol::parse("Dm7").unwrap();
        assert_eq!(cs.name, "Dm7");
        assert_eq!(cs.root_note.rel_pitch_to_string(), "D");
        assert_eq!(cs.chord_type_name, "m7");
    }
    #[test]
    fn test_parse_slash() {
        let cs = ChordSymbol::parse("Cm7/G").unwrap();
        assert!(cs.is_slash_chord());
    }
    #[test]
    fn test_parse_altered() {
        let cs = ChordSymbol::parse("F7b9").unwrap();
        assert_eq!(cs.chord_type_name, "7b9");
    }
}
