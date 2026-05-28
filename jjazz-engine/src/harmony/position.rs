use crate::harmony::time_signature::TimeSignature;

/// A musical position defined by bar index and beat within that bar.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub bar: i32,
    pub beat: f32,
}

impl Position {
    pub fn new(bar: i32, beat: f32) -> Self {
        assert!(bar >= 0 && beat >= 0.0);
        Self { bar, beat }
    }

    pub fn bar_zero() -> Self { Self { bar: 0, beat: 0.0 } }

    pub fn at_bar(bar: i32) -> Self { Self { bar, beat: 0.0 } }

    /// Convert to absolute beat position.
    pub fn to_absolute_beat(&self, ts: &TimeSignature) -> f32 {
        self.bar as f32 * ts.nb_natural_beats() + self.beat
    }

    /// Create from absolute beat position.
    pub fn from_absolute_beat(abs_beat: f32, ts: &TimeSignature) -> Self {
        let nb = ts.nb_natural_beats();
        let bar = (abs_beat / nb) as i32;
        let beat = abs_beat - bar as f32 * nb;
        Self { bar, beat }
    }

    pub fn beat_int(&self) -> i32 { self.beat as i32 }
    pub fn beat_frac(&self) -> f32 { self.beat - self.beat as i32 as f32 }
}
