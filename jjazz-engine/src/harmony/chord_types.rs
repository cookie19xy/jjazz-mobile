use std::collections::HashMap;
use std::sync::LazyLock;
use crate::harmony::chord_type::{ChordType, Family, NOT_PRESENT};
use crate::harmony::degree::Degree;

/// All built-in chord types.
pub static TYPES: LazyLock<HashMap<String, ChordType>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let types: Vec<ChordType> = vec![
        ChordType::new("", "", Family::Major, NOT_PRESENT, 0, NOT_PRESENT, 0, NOT_PRESENT, NOT_PRESENT),          // Major triad
        ChordType::new("m", "", Family::Minor, NOT_PRESENT, -1, NOT_PRESENT, 0, NOT_PRESENT, NOT_PRESENT),     // Minor triad
        ChordType::new("dim", "", Family::Diminished, NOT_PRESENT, -1, NOT_PRESENT, -1, NOT_PRESENT, NOT_PRESENT), // Dim
        ChordType::new("aug", "", Family::Major, NOT_PRESENT, 0, NOT_PRESENT, 1, NOT_PRESENT, NOT_PRESENT),     // Aug
        ChordType::new("sus", "", Family::Sus, NOT_PRESENT, NOT_PRESENT, 0, 0, NOT_PRESENT, NOT_PRESENT),       // Sus4
        ChordType::new("2", "", Family::Sus, 0, NOT_PRESENT, NOT_PRESENT, 0, NOT_PRESENT, NOT_PRESENT),         // Sus2

        ChordType::new("M7", "", Family::Major, NOT_PRESENT, 0, NOT_PRESENT, 0, NOT_PRESENT, 0),               // Maj7
        ChordType::new("7", "", Family::Seventh, NOT_PRESENT, 0, NOT_PRESENT, 0, NOT_PRESENT, -1),              // Dom7
        ChordType::new("m7", "", Family::Minor, NOT_PRESENT, -1, NOT_PRESENT, 0, NOT_PRESENT, -1),              // Min7
        ChordType::new("mM7", "", Family::Minor, NOT_PRESENT, -1, NOT_PRESENT, 0, NOT_PRESENT, 0),              // MinMaj7
        ChordType::new("dim7", "", Family::Diminished, NOT_PRESENT, -1, NOT_PRESENT, -1, 0, NOT_PRESENT),        // Dim7 (bb7=6)
        ChordType::new("m7b5", "", Family::Diminished, NOT_PRESENT, -1, NOT_PRESENT, -1, NOT_PRESENT, -1),      // Half-dim
        ChordType::new("7aug", "", Family::Seventh, NOT_PRESENT, 0, NOT_PRESENT, 1, NOT_PRESENT, -1),           // Aug7
        ChordType::new("M7aug", "", Family::Major, NOT_PRESENT, 0, NOT_PRESENT, 1, NOT_PRESENT, 0),             // AugMaj7
        ChordType::new("7sus", "", Family::Sus, NOT_PRESENT, NOT_PRESENT, 0, 0, NOT_PRESENT, -1),               // 7sus4
        ChordType::new("M7sus", "", Family::Sus, NOT_PRESENT, NOT_PRESENT, 0, 0, NOT_PRESENT, 0),               // Maj7sus4

        ChordType::new("6", "", Family::Major, NOT_PRESENT, 0, NOT_PRESENT, 0, 0, NOT_PRESENT),                 // Maj6
        ChordType::new("m6", "", Family::Minor, NOT_PRESENT, -1, NOT_PRESENT, 0, 0, NOT_PRESENT),               // Min6

        ChordType::new("M9", "", Family::Major, 0, 0, NOT_PRESENT, 0, NOT_PRESENT, 0),                          // Maj9
        ChordType::new("9", "", Family::Seventh, 0, 0, NOT_PRESENT, 0, NOT_PRESENT, -1),                        // Dom9
        ChordType::new("m9", "", Family::Minor, 0, -1, NOT_PRESENT, 0, NOT_PRESENT, -1),                        // Min9

        ChordType::new("7", "b9", Family::Seventh, -1, 0, NOT_PRESENT, 0, NOT_PRESENT, -1),                     // 7b9
        ChordType::new("7", "#9", Family::Seventh, 1, 0, NOT_PRESENT, 0, NOT_PRESENT, -1),                      // 7#9
        ChordType::new("7", "b5", Family::Seventh, NOT_PRESENT, 0, NOT_PRESENT, -1, NOT_PRESENT, -1),           // 7b5
        ChordType::new("7", "#5", Family::Seventh, NOT_PRESENT, 0, NOT_PRESENT, 1, NOT_PRESENT, -1),            // 7#5
        ChordType::new("7", "#11", Family::Seventh, NOT_PRESENT, 0, 1, 0, NOT_PRESENT, -1),                     // 7#11
        ChordType::new("7", "alt", Family::Seventh, -1, 0, NOT_PRESENT, -1, NOT_PRESENT, -1),                   // 7alt

        ChordType::new("M7", "#11", Family::Major, NOT_PRESENT, 0, 1, 0, NOT_PRESENT, 0),                       // Maj7#11
        ChordType::new("13", "", Family::Seventh, 0, 0, NOT_PRESENT, 0, 0, -1),                                 // Dom13
        ChordType::new("M13", "", Family::Major, 0, 0, NOT_PRESENT, 0, 0, 0),                                   // Maj13
        ChordType::new("m13", "", Family::Minor, 0, -1, NOT_PRESENT, 0, 0, -1),                                 // Min13
        ChordType::new("11", "", Family::Seventh, 0, 0, 0, 0, NOT_PRESENT, -1),                                 // Dom11
        ChordType::new("m11", "", Family::Minor, 0, -1, 0, 0, NOT_PRESENT, -1),                                 // Min11
    ];
    for ct in types {
        m.insert(ct.name.clone(), ct);
    }
    m
});

/// Lookup a chord type by name.
pub fn get(name: &str) -> Option<&ChordType> {
    TYPES.get(name)
}

/// Find a chord type matching a degree list exactly.
pub fn get_by_degrees(degs: &[Degree]) -> Option<&ChordType> {
    TYPES.values().find(|ct| ct.degrees == degs)
}
