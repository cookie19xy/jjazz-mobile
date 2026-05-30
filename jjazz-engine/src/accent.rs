use crate::phrase::{Phrase, NoteEvent};
use crate::harmony::TimeSignature;

/// Hold/Shot mode for accent processing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoldShotMode {
    Normal,    // Process hold/shot normally
    Extended,  // Only process extended hold/shot
    Ignore,    // Skip accent processing entirely
}

/// Chord accent type applied to a position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccentType {
    None,
    Hold,
    Shot,
    HoldShot,  // Both hold and shot
}

/// AccentProcessor: modifies phrases based on chord accent annotations.
/// This is what makes Hold/Shot/Extended work in JJazzLab.
pub struct AccentProcessor {
    pub hold_shot_mode: HoldShotMode,
}

impl AccentProcessor {
    pub fn new() -> Self {
        Self { hold_shot_mode: HoldShotMode::Normal }
    }

    /// Process a drum phrase: remove unwanted notes in hold/shot sections.
    /// In Hold sections, keep only short percussion. In Shot sections, add a crash.
    pub fn process_drums(&self, phrase: &mut Phrase, accents: &[(f32, AccentType)], ts: &TimeSignature) {
        if accents.is_empty() || phrase.is_empty() { return; }

        let bar_duration = ts.nb_natural_beats();

        // Collect note positions that should be removed
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, ne) in phrase.notes.iter().enumerate() {
            let bar_start = (ne.position / bar_duration).floor() * bar_duration;
            let rel_pos = ne.position - bar_start;

            // Find accent at this beat position
            for &(accent_beat, accent_type) in accents {
                let abs_accent = bar_start + accent_beat;
                if (ne.position - abs_accent).abs() < 0.01 {
                    match accent_type {
                        AccentType::Hold
                            // In Hold, keep only hi-hat/ride (short decay drums)
                            if ne.pitch != 42 && ne.pitch != 46 && ne.pitch != 51 => {
                                to_remove.push(i);
                            }
                        AccentType::Shot | AccentType::HoldShot
                            // In Shot, remove all except the first beat accent
                            if rel_pos > 0.1 => {
                                to_remove.push(i);
                            }
                        _ => {}
                    }
                }
            }
        }

        // Remove in reverse order
        to_remove.sort_unstable();
        for &idx in to_remove.iter().rev() {
            phrase.notes.remove(idx);
        }
    }

    /// Process a monophonic phrase (bass): extend notes in hold, add accent in shot.
    pub fn process_mono(&self, phrase: &mut Phrase, accents: &[(f32, AccentType)], ts: &TimeSignature) {
        if accents.is_empty() || phrase.is_empty() { return; }
        let bar_duration = ts.nb_natural_beats();

        // Collect data first to avoid borrow conflicts
        let notes_data: Vec<(f32, f32, u8, u8)> = phrase.notes.iter()
            .map(|n| (n.position, n.duration, n.pitch, n.velocity)).collect();
        let n = notes_data.len();
        phrase.notes.clear();

        for i in 0..n {
            let (pos, mut dur, pitch, mut vel) = notes_data[i];
            let bar_start = (pos / bar_duration).floor() * bar_duration;

            for &(accent_beat, accent_type) in accents {
                let abs = bar_start + accent_beat;
                if (pos - abs).abs() < 0.05 {
                    match accent_type {
                        AccentType::Shot | AccentType::HoldShot => {
                            dur = dur.min(0.15);
                            vel = (vel as u16 + 20).min(127) as u8;
                        }
                        AccentType::Hold => {
                            let next_pos = if i + 1 < n { notes_data[i + 1].0 }
                                else { bar_start + bar_duration };
                            dur = (next_pos - pos - 0.05).max(dur);
                        }
                        _ => {}
                    }
                }
            }
            if dur >= 0.02 { phrase.add(NoteEvent::new(pitch, dur, vel, pos)); }
        }
    }

    pub fn process_chord(&self, phrase: &mut Phrase, accents: &[(f32, AccentType)], ts: &TimeSignature) {
        self.process_mono(phrase, accents, ts);
        if phrase.is_empty() { return; }
        let bar_duration = ts.nb_natural_beats();
        let notes_pos: Vec<(f32, u8)> = phrase.notes.iter().map(|n| (n.position, n.pitch)).collect();
        let mut to_remove = Vec::new();

        for (_i, &(pos, _)) in notes_pos.iter().enumerate() {
            let bar_start = (pos / bar_duration).floor() * bar_duration;
            let rel_pos = pos - bar_start;
            for &(accent_beat, accent_type) in accents {
                if (rel_pos - accent_beat).abs() < 0.05
                    && matches!(accent_type, AccentType::Shot | AccentType::HoldShot)
                {
                    let mut group: Vec<usize> = (0..notes_pos.len())
                        .filter(|&j| (notes_pos[j].0 - pos).abs() < 0.02).collect();
                    if group.len() > 2 {
                        group.sort_by_key(|&j| notes_pos[j].1);
                        for &idx in &group[1..group.len()-1] { to_remove.push(idx); }
                    }
                    break;
                }
            }
        }
        to_remove.sort_unstable(); to_remove.dedup();
        for &idx in to_remove.iter().rev() { phrase.notes.remove(idx); }
    }
}

impl Default for AccentProcessor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_noop() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.5, 100, 0.0));
        let ap = AccentProcessor::new();
        let accents = vec![];
        ap.process_mono(&mut p, &accents, &TimeSignature::FOUR_FOUR);
        assert_eq!(p.len(), 1);
        assert_eq!(p.notes[0].pitch, 60);
    }

    #[test]
    fn test_shot_shortens_duration() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 1.0, 100, 0.0));
        let ap = AccentProcessor::new();
        let accents = vec![(0.0, AccentType::Shot)];
        ap.process_mono(&mut p, &accents, &TimeSignature::FOUR_FOUR);
        assert!(p.notes[0].duration < 0.5, "Shot should shorten, got {}", p.notes[0].duration);
    }

    #[test]
    fn test_hold_extends_duration() {
        let mut p = Phrase::new(0);
        p.add(NoteEvent::new(60, 0.3, 100, 0.0));
        p.add(NoteEvent::new(62, 0.3, 90, 2.0));
        let ap = AccentProcessor::new();
        let accents = vec![(0.0, AccentType::Hold)];
        ap.process_mono(&mut p, &accents, &TimeSignature::FOUR_FOUR);
        assert!(p.notes[0].duration > 1.0, "Hold should extend, got {}", p.notes[0].duration);
    }
}
