use crate::harmony::ChordSymbol;
use crate::harmony::degree::Degree;
use crate::style::RetriggerRule;

/// Adapt a source note to a target chord based on retrigger rules.
/// Returns the new pitch (or None if note should be stopped).
pub fn adapt_note(
    source_pitch: u8,
    source_chord: &ChordSymbol,
    target_chord: &ChordSymbol,
    rule: RetriggerRule,
) -> Option<u8> {
    match rule {
        RetriggerRule::Stop => None,
        RetriggerRule::Retrigger => Some(source_pitch),
        RetriggerRule::RetriggerToRoot => {
            let root = target_chord.root_note.relative_pitch();
            Some(closest_pitch(source_pitch, root))
        }
        RetriggerRule::PitchShift => {
            // Find what degree the source note is in the source chord
            let src_root_rp = source_chord.root_note.relative_pitch();
            let src_rel = (source_pitch as i32 - src_root_rp as i32).rem_euclid(12) as u8;

            // Find matching degree in target chord
            if let (Some(_src_ct), Some(tgt_ct)) = (source_chord.chord_type(), target_chord.chord_type()) {
                let src_degree = Degree::most_probable(src_rel);
                let tgt_degree = tgt_ct.fit_degree(src_degree).unwrap_or(src_degree);
                let tgt_root_rp = target_chord.root_note.relative_pitch();
                let new_rp = (tgt_root_rp + tgt_degree.pitch()) % 128;
                Some(closest_pitch(source_pitch, new_rp))
            } else {
                // Fallback: just transpose by root difference
                let diff = target_chord.root_note.relative_pitch() as i32
                    - source_chord.root_note.relative_pitch() as i32;
                Some((source_pitch as i32 + diff).clamp(0, 127) as u8)
            }
        }
        RetriggerRule::PitchShiftToRoot => {
            // Shift toward target chord root
            let target_root = target_chord.root_note.relative_pitch();
            let src_root = source_chord.root_note.relative_pitch();
            let diff = target_root as i32 - src_root as i32;
            Some((source_pitch as i32 + diff).clamp(0, 127) as u8)
        }
        RetriggerRule::NoteGenerator => {
            // Generate a note from target chord tones
            if let Some(ct) = target_chord.chord_type() {
                let deg = ct.degrees.first().copied().unwrap_or(Degree::Root);
                let rp = target_chord.root_note.relative_pitch() + deg.pitch();
                Some(closest_pitch(source_pitch, rp))
            } else {
                Some(closest_pitch(source_pitch, target_chord.root_note.relative_pitch()))
            }
        }
    }
}

/// Find the closest pitch to `reference` that has relative pitch `target_rp`.
fn closest_pitch(reference: u8, target_rp: u8) -> u8 {
    let ref_octave = reference / 12;
    let candidates = [
        (ref_octave as i32 - 1) * 12 + target_rp as i32,
        ref_octave as i32 * 12 + target_rp as i32,
        (ref_octave as i32 + 1) * 12 + target_rp as i32,
    ];
    candidates.iter()
        .map(|&c| c.clamp(0, 127) as u8)
        .min_by_key(|&p| (p as i32 - reference as i32).abs())
        .unwrap_or(reference)
}

/// Adapt a phrase from source chord to target chord.
pub fn adapt_phrase(
    source_notes: &[(u8, f32, u8, f32)], // (pitch, dur, vel, pos) - relative to bar
    source_chord: &ChordSymbol,
    target_chord: &ChordSymbol,
    rule: RetriggerRule,
) -> Vec<(u8, f32, u8, f32)> {
    source_notes.iter().filter_map(|&(pitch, dur, vel, pos)| {
        adapt_note(pitch, source_chord, target_chord, rule)
            .map(|new_pitch| (new_pitch, dur, vel, pos))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retrigger_same_chord() {
        let c = ChordSymbol::parse("C").unwrap();
        let pitch = adapt_note(60, &c, &c, RetriggerRule::Retrigger);
        assert_eq!(pitch, Some(60));
    }
    #[test]
    fn test_stop() {
        let c = ChordSymbol::parse("C").unwrap();
        assert_eq!(adapt_note(60, &c, &c, RetriggerRule::Stop), None);
    }
    #[test]
    fn test_retrigger_to_root() {
        let src = ChordSymbol::parse("C").unwrap();
        let tgt = ChordSymbol::parse("G").unwrap();
        // C4(60) → G root. Relative pitch of G is 7.
        let result = adapt_note(60, &src, &tgt, RetriggerRule::RetriggerToRoot);
        // Should be near G (67 = G4, or 55 = G3)
        assert!(result.unwrap() % 12 == 7);
    }
}
