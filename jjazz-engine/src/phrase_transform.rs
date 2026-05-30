use crate::phrase::{Phrase, NoteEvent};
use crate::harmony::{ChordSymbol, TimeSignature};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════
//  SimpleChordSequence: beat-level chord cells
// ═══════════════════════════════════════════════════════════

/// A chord cell in a sequence: which chord is active at which beat.
#[derive(Debug, Clone)]
pub struct ChordCell {
    pub chord: ChordSymbol,
    pub start_beat: f32,
    pub end_beat: f32,
}

/// A sequence of chords with precise beat positions.
#[derive(Debug, Clone)]
pub struct SimpleChordSequence {
    pub cells: Vec<ChordCell>,
    pub total_beats: f32,
}

impl SimpleChordSequence {
    pub fn new(chords: &[ChordSymbol], beats_per_chord: f32) -> Self {
        let cells: Vec<ChordCell> = chords.iter().enumerate().map(|(i, cs)| {
            let start = i as f32 * beats_per_chord;
            ChordCell { chord: cs.clone(), start_beat: start, end_beat: start + beats_per_chord }
        }).collect();
        let total = cells.last().map(|c| c.end_beat).unwrap_or(0.0);
        Self { cells, total_beats: total }
    }

    /// Find which chord cell a beat position falls into.
    pub fn chord_at(&self, beat: f32) -> Option<&ChordCell> {
        self.cells.iter().find(|c| beat >= c.start_beat && beat < c.end_beat)
    }

    /// Is this beat position near a chord boundary? (within `window` beats)
    pub fn is_boundary(&self, beat: f32, window: f32) -> Option<&ChordCell> {
        self.cells.iter().find(|c| (beat - c.start_beat).abs() < window && c.start_beat > 0.0)
    }
}

// ═══════════════════════════════════════════════════════════
//  GridChordContext: grid-based chord tracking
// ═══════════════════════════════════════════════════════════

/// A grid cell for tracking chord activity per subdivision.
#[derive(Debug, Clone)]
pub struct GridChordContext {
    pub grid: Vec<Option<usize>>, // cell → chord index (None = no chord)
    pub cells_per_beat: usize,
    pub chord_cells: Vec<usize>,  // which grid cells have chord boundaries
    pub beats_offset: f32,
}

impl GridChordContext {
    /// Build a grid from a chord sequence, dividing each beat into `cells_per_beat` cells.
    pub fn build(seq: &SimpleChordSequence, cells_per_beat: usize, pre_window: f32) -> Self {
        let total_cells = ((seq.total_beats + pre_window) * cells_per_beat as f32).ceil() as usize + 1;
        let mut grid = vec![None; total_cells];
        let mut chord_cells = Vec::new();

        for (ci, cell) in seq.cells.iter().enumerate() {
            let start_cell = ((cell.start_beat + pre_window) * cells_per_beat as f32).floor() as usize;
            let end_cell = ((cell.end_beat + pre_window) * cells_per_beat as f32).ceil() as usize;

            for c in start_cell..end_cell.min(total_cells) {
                grid[c] = Some(ci);
            }
            if start_cell < total_cells {
                chord_cells.push(start_cell);
            }
        }

        Self { grid, cells_per_beat, chord_cells, beats_offset: 0.0 }
    }

    pub fn chord_index_at_cell(&self, cell: usize) -> Option<usize> {
        self.grid.get(cell).copied().flatten()
    }
}

// ═══════════════════════════════════════════════════════════
//  SwingTransform: swing feel (shifts off-beat 8th notes)
// ═══════════════════════════════════════════════════════════

/// Apply swing feel: delays off-beat 8th notes.
/// `swing_amount`: 0.0 = straight, 0.3 = light swing, 0.66 = heavy triplet swing.
pub fn apply_swing(phrase: &mut Phrase, swing_amount: f32, ts: &TimeSignature) {
    if swing_amount <= 0.0 || phrase.is_empty() { return; }

    let beat_duration = ts.nb_natural_beats();
    let half_beat = beat_duration / (ts.numerator as f32 * 2.0);

    for ne in &mut phrase.notes {
        let beat_pos = ne.position % beat_duration;
        let eighth_index = (beat_pos / half_beat).round() as i32;

        // Shift odd 8th notes (off-beats: index 1, 3, 5, 7...)
        if eighth_index % 2 == 1 {
            let shift = half_beat * swing_amount;
            ne.position += shift;
            ne.duration = (ne.duration - shift).max(0.02);
        }
    }
    phrase.sort();
}

// ═══════════════════════════════════════════════════════════
//  Drum Transforms: HH↔Ride, Snare↔Rim, Open HH
// ═══════════════════════════════════════════════════════════

/// GM drum note numbers.
pub const GM_KICK: u8 = 36;
pub const GM_SNARE: u8 = 38;
pub const GM_RIM: u8 = 37;
pub const GM_HIHAT_CLOSED: u8 = 42;
pub const GM_HIHAT_OPEN: u8 = 46;
pub const GM_HIHAT_PEDAL: u8 = 44;
pub const GM_RIDE: u8 = 51;
pub const GM_CRASH: u8 = 49;

/// Replace hi-hat with ride cymbal.
pub fn hihat_to_ride(phrase: &mut Phrase, amount: f32) {
    for ne in &mut phrase.notes {
        if ne.pitch == GM_HIHAT_CLOSED && rand_f32() < amount {
            ne.pitch = GM_RIDE;
        }
    }
}

/// Replace ride with hi-hat.
pub fn ride_to_hihat(phrase: &mut Phrase, amount: f32) {
    for ne in &mut phrase.notes {
        if ne.pitch == GM_RIDE && rand_f32() < amount {
            ne.pitch = GM_HIHAT_CLOSED;
        }
    }
}

/// Apply open hi-hat effect: replace some closed HH with open HH.
pub fn open_hihat(phrase: &mut Phrase, amount: f32) {
    for ne in &mut phrase.notes {
        if ne.pitch == GM_HIHAT_CLOSED && rand_f32() < amount {
            ne.pitch = GM_HIHAT_OPEN;
        }
    }
}

/// Replace snare with rim shot.
pub fn snare_to_rim(phrase: &mut Phrase, amount: f32) {
    for ne in &mut phrase.notes {
        if ne.pitch == GM_SNARE && rand_f32() < amount {
            ne.pitch = GM_RIM;
        }
    }
}

/// Replace rim shot with snare.
pub fn rim_to_snare(phrase: &mut Phrase, amount: f32) {
    for ne in &mut phrase.notes {
        if ne.pitch == GM_RIM && rand_f32() < amount {
            ne.pitch = GM_SNARE;
        }
    }
}

/// Add crash cymbal on beat 1 of each bar.
pub fn add_crash_on_downbeat(phrase: &mut Phrase, ts: &TimeSignature, amount: f32) {
    if amount <= 0.0 { return; }
    let bar_duration = ts.nb_natural_beats();
    let num_bars = phrase.notes.iter().map(|n| n.position).fold(0.0f32, f32::max) / bar_duration;

    for bar in 0..=(num_bars as i32) {
        let pos = bar as f32 * bar_duration;
        if rand_f32() < amount {
            phrase.add(NoteEvent::new(GM_CRASH, 0.5, 100, pos));
        }
    }
    phrase.sort();
}

fn rand_f32() -> f32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut h = RandomState::new().build_hasher();
    h.write_u32(0);
    (h.finish() % 1000) as f32 / 1000.0
}

/// Full drum transform pipeline for a style variation.
pub fn drums_transform(phrase: &mut Phrase, ts: &TimeSignature, profile: &DrumProfile) {
    hihat_to_ride(phrase, profile.hihat_to_ride);
    ride_to_hihat(phrase, profile.ride_to_hihat);
    open_hihat(phrase, profile.open_hihat);
    snare_to_rim(phrase, profile.snare_to_rim);
    rim_to_snare(phrase, profile.rim_to_snare);
    add_crash_on_downbeat(phrase, ts, profile.crash_on_1);
}

#[derive(Debug, Clone)]
pub struct DrumProfile {
    pub hihat_to_ride: f32,
    pub ride_to_hihat: f32,
    pub open_hihat: f32,
    pub snare_to_rim: f32,
    pub rim_to_snare: f32,
    pub crash_on_1: f32,
}

impl Default for DrumProfile {
    fn default() -> Self {
        Self {
            hihat_to_ride: 0.0,
            ride_to_hihat: 0.0,
            open_hihat: 0.1,
            snare_to_rim: 0.0,
            rim_to_snare: 0.0,
            crash_on_1: 0.0,
        }
    }
}

impl DrumProfile {
    pub fn jazz() -> Self {
        Self { hihat_to_ride: 0.9, open_hihat: 0.15, crash_on_1: 0.3, ..Default::default() }
    }
    pub fn bossa() -> Self {
        Self { snare_to_rim: 0.8, open_hihat: 0.05, ..Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_sequence() {
        let cs = vec![
            ChordSymbol::parse("C").unwrap(),
            ChordSymbol::parse("G").unwrap(),
        ];
        let seq = SimpleChordSequence::new(&cs, 8.0);
        assert_eq!(seq.cells.len(), 2);
        assert_eq!(seq.total_beats, 16.0);
        assert!(seq.chord_at(4.0).is_some());
        assert!(seq.chord_at(9.0).is_some());
        assert!(seq.chord_at(16.0).is_none());
    }

    #[test]
    fn test_swing_shifts_offbeat() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.3, 100, 0.0)); // on-beat
        p.add(NoteEvent::new(62, 0.3, 90, 0.5));  // off-beat
        let orig_off = p.notes[1].position;
        apply_swing(&mut p, 0.5, &TimeSignature::FOUR_FOUR);
        assert!(p.notes[1].position > orig_off);
        assert!((p.notes[0].position - 0.0).abs() < 0.001, "On-beat should not shift");
    }

    #[test]
    fn test_hihat_to_ride() {
        let mut p = Phrase::new(9);
        for _ in 0..50 { p.add(NoteEvent::new(GM_HIHAT_CLOSED, 0.1, 70, 0.0)); }
        let orig = p.notes.iter().filter(|n| n.pitch == GM_HIHAT_CLOSED).count();
        hihat_to_ride(&mut p, 1.0);
        assert_eq!(p.notes.iter().filter(|n| n.pitch == GM_HIHAT_CLOSED).count(), 0);
        assert_eq!(p.notes.iter().filter(|n| n.pitch == GM_RIDE).count(), orig);
    }

    #[test]
    fn test_grid_context() {
        let cs = vec![ChordSymbol::parse("C").unwrap(), ChordSymbol::parse("G").unwrap()];
        let seq = SimpleChordSequence::new(&cs, 4.0);
        let gctx = GridChordContext::build(&seq, 4, 0.0);
        assert!(gctx.grid.len() > 0);
        assert!(gctx.chord_cells.len() > 0);
    }

    #[test]
    fn test_drum_profiles() {
        let j = DrumProfile::jazz();
        assert!(j.hihat_to_ride > 0.5);
        let b = DrumProfile::bossa();
        assert!(b.snare_to_rim > 0.5);
    }
}
