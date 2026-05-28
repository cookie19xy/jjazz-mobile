/// Time signature (e.g. 4/4, 3/4, 6/8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl TimeSignature {
    pub const FOUR_FOUR: Self = Self { numerator: 4, denominator: 4 };
    pub const THREE_FOUR: Self = Self { numerator: 3, denominator: 4 };
    pub const TWO_FOUR: Self = Self { numerator: 2, denominator: 4 };
    pub const FIVE_FOUR: Self = Self { numerator: 5, denominator: 4 };
    pub const SIX_EIGHT: Self = Self { numerator: 6, denominator: 8 };

    /// Duration of one natural beat in 4/4-time beats.
    pub fn natural_beat(&self) -> f32 {
        if self.denominator == 8 { 1.5 } else { 1.0 }
    }

    /// Total natural beats per bar.
    pub fn nb_natural_beats(&self) -> f32 {
        self.numerator as f32 * self.natural_beat()
    }

    /// Half-bar beat position.
    pub fn half_bar_beat(&self) -> f32 {
        (self.numerator as f32 * self.natural_beat()) / 2.0
    }

    /// Check if a beat position is valid within this bar.
    pub fn check_beat(&self, beat: f32) -> bool {
        beat >= 0.0 && beat < self.nb_natural_beats()
    }
}

impl std::fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}
