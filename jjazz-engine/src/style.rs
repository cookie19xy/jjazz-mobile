use crate::harmony::degree::Degree;

/// Voice types in a Yamaha-style rhythm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccType {
    SubRhythm,
    Rhythm,
    Bass,
    Chord1,
    Chord2,
    Pad,
    Phrase1,
    Phrase2,
}

impl AccType {
    /// GM program number for this voice type.
    pub fn gm_program(&self) -> u8 {
        match self {
            AccType::SubRhythm | AccType::Rhythm => 0,  // drums (channel 10)
            AccType::Bass => 33,     // Acoustic Bass
            AccType::Chord1 => 26,   // Jazz Guitar
            AccType::Chord2 => 1,    // Piano
            AccType::Pad => 50,      // Slow Strings
            AccType::Phrase1 => 62,  // Synth Brass
            AccType::Phrase2 => 2,   // Bright Piano
        }
    }

    /// Default "preferred" chord degrees for this voice.
    pub fn preferred_degrees(&self) -> Vec<Degree> {
        match self {
            AccType::Bass => vec![Degree::Root, Degree::Fifth, Degree::SeventhFlat, Degree::Third],
            AccType::Chord1 => vec![Degree::Third, Degree::Seventh, Degree::Root, Degree::Fifth],
            AccType::Chord2 => vec![Degree::Root, Degree::Third, Degree::Fifth, Degree::Seventh],
            AccType::Pad => vec![Degree::Third, Degree::Seventh, Degree::Root],
            AccType::Phrase1 | AccType::Phrase2 => vec![Degree::Root, Degree::Third, Degree::Fifth, Degree::Seventh],
            _ => vec![Degree::Root],
        }
    }

    pub fn channel(&self) -> u8 {
        match self {
            AccType::SubRhythm | AccType::Rhythm => 9, // drums on ch10 (0-indexed 9)
            AccType::Bass => 0,
            AccType::Chord1 => 1,
            AccType::Chord2 => 2,
            AccType::Pad => 3,
            AccType::Phrase1 => 4,
            AccType::Phrase2 => 5,
        }
    }

    pub fn is_drums(&self) -> bool {
        matches!(self, AccType::SubRhythm | AccType::Rhythm)
    }
}

/// How notes adapt when chord changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetriggerRule {
    Stop,               // Note stops
    PitchShift,         // Shift by chord interval difference
    PitchShiftToRoot,   // Shift to new chord root
    Retrigger,          // Replay same pitch
    RetriggerToRoot,    // Replay at new chord root pitch
    NoteGenerator,      // Generate new note based on chord
}

/// Per-channel chord adaptation settings.
#[derive(Debug, Clone)]
pub struct ChannelSettings {
    pub retrigger_rule: RetriggerRule,
    pub bass_on: bool,
    pub note_low: u8,
    pub note_high: u8,
    pub chord_root_upper_limit: u8,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            retrigger_rule: RetriggerRule::Retrigger,
            bass_on: false,
            note_low: 0,
            note_high: 127,
            chord_root_upper_limit: 90,
        }
    }
}

/// A musical style section (e.g. "Bossa Nova Variation A").
#[derive(Debug, Clone)]
pub struct StylePart {
    pub name: String,
    pub time_numerator: u8,
    pub time_denominator: u8,
    pub channel_settings: Vec<ChannelSettings>, // one per AccType
}

/// A complete musical style with multiple variations.
#[derive(Debug, Clone)]
pub struct Style {
    pub name: String,
    pub parts: Vec<StylePart>, // Variation A, B, C, D
}

impl Style {
    /// Create predefined Bossanova style.
    pub fn bossanova() -> Self {
        let bass_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::RetriggerToRoot,
            bass_on: false,
            note_low: 24,
            note_high: 60,
            chord_root_upper_limit: 80,
        };
        let chord1_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::RetriggerToRoot,
            bass_on: false,
            note_low: 40,
            note_high: 80,
            chord_root_upper_limit: 85,
        };
        let chord2_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::PitchShift,
            bass_on: false,
            note_low: 50,
            note_high: 90,
            chord_root_upper_limit: 85,
        };
        let pad_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::PitchShiftToRoot,
            bass_on: false,
            note_low: 48,
            note_high: 84,
            chord_root_upper_limit: 90,
        };

        let settings = vec![
            ChannelSettings::default(), // SubRhythm - drums
            ChannelSettings::default(), // Rhythm - drums
            bass_cfg,                   // Bass
            chord1_cfg,                 // Chord1
            chord2_cfg,                 // Chord2
            pad_cfg,                    // Pad
            ChannelSettings::default(), // Phrase1
            ChannelSettings::default(), // Phrase2
        ];

        let part_a = StylePart {
            name: "Bossa A".into(),
            time_numerator: 4,
            time_denominator: 4,
            channel_settings: settings,
        };

        Style {
            name: "Bossa Nova".into(),
            parts: vec![part_a],
        }
    }

    /// Create predefined Swing style.
    pub fn swing() -> Self {
        let bass_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::RetriggerToRoot,
            bass_on: false,
            note_low: 24,
            note_high: 60,
            chord_root_upper_limit: 80,
        };
        let chord1_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::RetriggerToRoot,
            bass_on: false,
            note_low: 40,
            note_high: 80,
            chord_root_upper_limit: 85,
        };
        let chord2_cfg = ChannelSettings {
            retrigger_rule: RetriggerRule::PitchShift,
            bass_on: false,
            note_low: 50,
            note_high: 90,
            chord_root_upper_limit: 85,
        };
        let settings = vec![
            ChannelSettings::default(),
            ChannelSettings::default(),
            bass_cfg,
            chord1_cfg,
            chord2_cfg,
            ChannelSettings::default(),
            ChannelSettings::default(),
            ChannelSettings::default(),
        ];
        let part = StylePart {
            name: "Swing A".into(),
            time_numerator: 4,
            time_denominator: 4,
            channel_settings: settings,
        };
        Style {
            name: "Swing".into(),
            parts: vec![part],
        }
    }
}
