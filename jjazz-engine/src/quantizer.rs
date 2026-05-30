use crate::harmony::{Position, TimeSignature};

/// Quantization grid types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantization {
    Off,
    Beat,
    HalfBeat,
    ThirdBeat,
    QuarterBeat,
}

fn quantize_impl(frac: f32, q_points: &[f32], strength: f32) -> f32 {
    if q_points.len() == 1 {
        let target = q_points[0];
        return if (frac - target).abs() <= 0.5 { target } else { frac };
    }
    for i in 0..q_points.len() - 1 {
        if frac == q_points[i] || frac == q_points[i + 1] { break; }
        if frac < q_points[i + 1] {
            let (lower, upper) = (q_points[i], q_points[i + 1]);
            let step = (upper - lower) / 2.0 * strength;
            let new_frac = if frac - lower < upper - frac {
                (frac - step).max(lower)
            } else {
                (frac + step).min(upper)
            };
            return if (new_frac - lower).abs() <= 0.01 { lower } else if (upper - new_frac).abs() <= 0.01 { upper } else { new_frac };
        }
    }
    frac
}

pub fn quantize(q: Quantization, pos: &Position, ts: &TimeSignature, strength: f32, max_bar: i32) -> Position {
    match q {
        Quantization::Off => *pos,
        Quantization::Beat => {
            let beat_int = pos.beat as i32 as f32;
            let frac = pos.beat - beat_int;
            let new_frac = quantize_impl(frac, &[0.0], strength);
            let new_beat = beat_int + new_frac;
            if ts.check_beat(new_beat) { Position::new(pos.bar, new_beat) }
            else if pos.bar < max_bar { Position::at_bar(pos.bar + 1) }
            else { Position::new(pos.bar, beat_int) }
        }
        Quantization::HalfBeat => {
            let q_pts = [0.0, 0.5];
            let beat_int = pos.beat as i32 as f32;
            let frac = pos.beat - beat_int;
            Position::new(pos.bar, beat_int + quantize_impl(frac, &q_pts, strength))
        }
        Quantization::QuarterBeat => {
            let q_pts = [0.0, 0.25, 0.5, 0.75];
            let beat_int = pos.beat as i32 as f32;
            let frac = pos.beat - beat_int;
            Position::new(pos.bar, beat_int + quantize_impl(frac, &q_pts, strength))
        }
        Quantization::ThirdBeat => {
            let q_pts = [0.0, 0.33333, 0.66667];
            let beat_int = pos.beat as i32 as f32;
            let frac = pos.beat - beat_int;
            Position::new(pos.bar, beat_int + quantize_impl(frac, &q_pts, strength))
        }
    }
}
