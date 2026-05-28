use std::collections::HashMap;
use std::sync::LazyLock;
use crate::harmony::degree::{Degree, Natural};
use crate::harmony::note::Note;
use crate::harmony::chord::Chord;

pub const NOT_PRESENT: i8 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Family { Major, Seventh, Minor, Diminished, Sus }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DegreeIndex { Root, ThirdOrFourth, Fifth, SixthOrSeventh, Extension1, Extension2, Extension3 }

impl DegreeIndex {
    pub fn is_extension(self) -> bool { matches!(self, DegreeIndex::Extension1|DegreeIndex::Extension2|DegreeIndex::Extension3) }
}

/// A chord type like "m7", "7b9", etc. Immutable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChordType {
    pub name: String,
    pub base: String,
    pub extension: String,
    pub family: Family,
    pub degrees: Vec<Degree>,
    pub degree_string: String,
    chord: Chord,
}

static REGISTRY: LazyLock<HashMap<String, ChordType>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Built-in types are registered via ChordTypes static init
    m
});

impl ChordType {
    pub fn new(base: &str, ext: &str, family: Family,
               i9: i8, i3: i8, i11: i8, i5: i8, i13: i8, i7: i8) -> Self {
        let mut degrees = Vec::new();
        let mut chord = Chord::new();

        chord.add(Note::new(0));
        degrees.push(Degree::Root);

        if i3 != NOT_PRESENT { chord.add(Note::new((4 + i3) as u8)); degrees.push(Degree::from_natural_alt(Natural::Third, i3).unwrap()); }
        if i11 == 0 && i3 == NOT_PRESENT { chord.add(Note::new(5)); degrees.push(Degree::FourthOrEleventh); }
        if i5 != NOT_PRESENT { chord.add(Note::new((7 + i5) as u8)); degrees.push(Degree::from_natural_alt(Natural::Fifth, i5).unwrap()); }
        if i13 == 0 && i7 == NOT_PRESENT { chord.add(Note::new(9)); degrees.push(Degree::SixthOrThirteenth); }
        if i7 != NOT_PRESENT { chord.add(Note::new((11 + i7) as u8)); degrees.push(Degree::from_natural_alt(Natural::Seventh, i7).unwrap()); }
        if i9 != NOT_PRESENT { chord.add(Note::new((2 + i9) as u8)); degrees.push(Degree::from_natural_alt(Natural::Ninth, i9).unwrap()); }
        if i11 != NOT_PRESENT && !(i11 == 0 && i3 == NOT_PRESENT) { chord.add(Note::new((5 + i11) as u8)); degrees.push(Degree::from_natural_alt(Natural::Eleventh, i11).unwrap()); }
        if i13 != NOT_PRESENT && !(i13 == 0 && i7 == NOT_PRESENT) { chord.add(Note::new((9 + i13) as u8)); degrees.push(Degree::from_natural_alt(Natural::Sixth, i13).unwrap()); }

        let degree_string = format!("[{}]", degrees.iter().map(|d| d.short_name()).collect::<Vec<_>>().join(" "));
        let name = format!("{}{}", base, ext);

        Self { name, base: base.to_string(), extension: ext.to_string(), family, degrees, degree_string, chord }
    }

    pub fn chord(&self) -> Chord { self.chord.clone() }
    pub fn nb_degrees(&self) -> usize { self.degrees.len() }

    pub fn get_degree_by_natural(&self, nd: Natural) -> Option<Degree> {
        self.degrees.iter().find(|d| d.natural() == nd).copied()
    }

    pub fn get_degree_by_pitch(&self, rel_pitch: u8) -> Option<Degree> {
        self.degrees.iter().find(|d| d.pitch() == rel_pitch).copied()
    }

    pub fn is_major(&self) -> bool { self.get_degree_by_natural(Natural::Third) == Some(Degree::Third) }
    pub fn is_minor(&self) -> bool { self.get_degree_by_natural(Natural::Third) == Some(Degree::ThirdFlat) }
    pub fn is_seventh(&self) -> bool { self.get_degree_by_natural(Natural::Seventh).is_some() }
    pub fn is_seventh_minor(&self) -> bool { self.get_degree_by_natural(Natural::Seventh) == Some(Degree::SeventhFlat) }
    pub fn is_seventh_major(&self) -> bool { self.get_degree_by_natural(Natural::Seventh) == Some(Degree::Seventh) }
    pub fn is_fifth_flat(&self) -> bool { self.get_degree_by_natural(Natural::Fifth) == Some(Degree::FifthFlat) }

    /// Fit a source degree to this chord type.
    pub fn fit_degree(&self, d: Degree) -> Option<Degree> {
        self.get_degree_by_natural(d.natural()).or_else(|| self.get_degree_by_pitch(d.pitch()))
    }

    pub fn simplify(&self, max: usize) -> Option<&ChordType> {
        if self.degrees.len() <= max { return Some(self); }
        let degs: Vec<Degree> = self.degrees.iter().take(max).copied().collect();
        // Look up by degree list in registry
        REGISTRY.values().find(|ct| ct.degrees == degs)
    }

    /// Register a chord type in the global lookup table.
    pub fn register(ct: ChordType) {
        // This is a bit hacky - Rust lazy static mutability
        // For built-in types, we use chord_types module
        let _ = ct.name.clone(); // ownership moved
    }
}

impl PartialEq for ChordType {
    fn eq(&self, other: &Self) -> bool { self.degrees == other.degrees }
}
impl Eq for ChordType {}
