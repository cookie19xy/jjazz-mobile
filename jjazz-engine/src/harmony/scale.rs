use crate::harmony::degree::Degree;
use crate::harmony::note::Note;

/// A musical scale defined by ascending degrees.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Scale {
    pub name: String,
    pub degrees: Vec<Degree>,
}

impl Scale {
    pub fn new(name: &str, degrees: Vec<Degree>) -> Self {
        assert!(!degrees.is_empty() && degrees[0] == Degree::Root);
        Self { name: name.into(), degrees }
    }

    pub fn size(&self) -> usize { self.degrees.len() }

    /// Get notes of this scale starting from root_pitch.
    pub fn notes(&self, root_pitch: u8) -> Vec<Note> {
        self.degrees.iter().map(|d| Note::new(root_pitch + d.pitch())).collect()
    }
}

/// Predefined standard scales.
pub mod standard {
    use super::*;
    use Degree::*;
    macro_rules! scale { ($n:expr, $($d:ident),+) => { Scale::new($n, vec![$($d),+]) }; }

    pub fn major() -> Scale { scale!("Major", Root, Ninth, Third, FourthOrEleventh, Fifth, SixthOrThirteenth, Seventh) }
    pub fn dorian() -> Scale { scale!("Dorian", Root, Ninth, ThirdFlat, FourthOrEleventh, Fifth, SixthOrThirteenth, SeventhFlat) }
    pub fn mixolydian() -> Scale { scale!("Mixolydian", Root, Ninth, Third, FourthOrEleventh, Fifth, SixthOrThirteenth, SeventhFlat) }
    pub fn aeolian() -> Scale { scale!("Aeolian", Root, Ninth, ThirdFlat, FourthOrEleventh, Fifth, SixthOrThirteenth, SeventhFlat) }
    pub fn major_pentatonic() -> Scale { scale!("Major Pentatonic", Root, Ninth, Third, Fifth, SixthOrThirteenth) }
    pub fn minor_pentatonic() -> Scale { scale!("Minor Pentatonic", Root, ThirdFlat, FourthOrEleventh, Fifth, SeventhFlat) }
    pub fn blues() -> Scale { scale!("Blues", Root, ThirdFlat, FourthOrEleventh, FifthFlat, Fifth, SeventhFlat) }
}
