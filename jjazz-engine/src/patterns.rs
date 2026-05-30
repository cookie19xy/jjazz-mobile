use crate::harmony::ChordSymbol;
use crate::phrase::NoteEvent;

/// A single MIDI note template: (pitch_offset_from_root, position_in_beats, duration, velocity)
type NoteTemplate = (i32, f32, f32, u8);

/// A musical pattern that can be adapted to any chord.
pub struct Pattern {
    pub templates: Vec<NoteTemplate>,
}

impl Pattern {
    pub fn render(&self, root: u8, bar_start: f32, octave_offset: i32) -> Vec<NoteEvent> {
        self.templates.iter().map(|&(offset, pos, dur, vel)| {
            let pitch = (root as i32 + octave_offset + offset).clamp(0, 127) as u8;
            NoteEvent::new(pitch, dur, vel, bar_start + pos)
        }).collect()
    }
}

// ─── DRUM PATTERNS ────────────────────────────────────────

/// Bossa Nova drum groove (kick 36, rim 37, snare 38, HH 42, ride 51)
pub fn drum_bossa(bar_start: f32) -> Vec<NoteEvent> {
    let mut notes = Vec::new();
    // Cross-stick rim on 2 and 4 (Bossa signature)
    notes.push(NoteEvent::new(37, 0.2, 95, bar_start + 1.0));
    notes.push(NoteEvent::new(37, 0.2, 95, bar_start + 3.0));
    // Kick pattern: 1, 2+, 3+
    notes.push(NoteEvent::new(36, 0.25, 110, bar_start + 0.0));
    notes.push(NoteEvent::new(36, 0.2, 100, bar_start + 1.5));
    notes.push(NoteEvent::new(36, 0.2, 105, bar_start + 2.5));
    // Hi-hat on 8th notes (standard Bossa groove)
    let hh_vels = [65, 50, 68, 52, 62, 48, 65, 50];
    for i in 0..8 {
        notes.push(NoteEvent::new(42, 0.08, hh_vels[i], bar_start + i as f32 * 0.5));
    }
    notes
}

/// Standard swing drum groove
pub fn drum_swing(bar_start: f32) -> Vec<NoteEvent> {
    let mut notes = Vec::new();
    notes.push(NoteEvent::new(36, 0.3, 115, bar_start + 0.0));
    notes.push(NoteEvent::new(42, 0.1, 60, bar_start + 0.0));
    notes.push(NoteEvent::new(38, 0.2, 105, bar_start + 1.0));
    notes.push(NoteEvent::new(42, 0.1, 58, bar_start + 1.0));
    notes.push(NoteEvent::new(36, 0.3, 105, bar_start + 2.0));
    notes.push(NoteEvent::new(42, 0.1, 60, bar_start + 2.0));
    notes.push(NoteEvent::new(38, 0.2, 108, bar_start + 3.0));
    notes.push(NoteEvent::new(42, 0.1, 55, bar_start + 3.0));
    // Ride cymbal
    for i in 0..8 {
        notes.push(NoteEvent::new(51, 0.1, 62, bar_start + i as f32 * 0.5));
    }
    notes
}

// ─── BASS PATTERNS ────────────────────────────────────────

/// Walking bass: 4 beats per bar, chromatic approach to next chord
pub fn bass_walking(root: u8, next_root: u8, bar_start: f32, is_last_bar: bool) -> Vec<NoteEvent> {
    let mut notes = Vec::new();
    let r = root as i32;
    let nr = next_root as i32;

    // Beat 1: root (strong)
    notes.push(NoteEvent::new(root, 0.8, 115, bar_start));
    // Beat 2: chord tone (third or fifth)
    notes.push(NoteEvent::new((r + 4).clamp(0, 127) as u8, 0.5, 95, bar_start + 1.0));
    // Beat 3: approach to fifth or chromatic
    let b3 = if (nr - r).abs() > 5 { r + 7 } else { r + 8 };
    notes.push(NoteEvent::new(b3.clamp(0, 127) as u8, 0.5, 90, bar_start + 2.0));
    // Beat 4: chromatic approach to next bar's root
    let b4 = if !is_last_bar {
        if nr > r { nr - 1 } else if nr < r { nr + 1 } else { nr }
    } else {
        r + 4 // fifth of current chord if last bar
    };
    notes.push(NoteEvent::new(b4.clamp(0, 127) as u8, 0.5, 85, bar_start + 3.0));
    notes
}

/// Bossa bass pattern (root-fifth-root-fifth, characteristic rhythm)
pub fn bass_bossa(root: u8, bar_start: f32) -> Vec<NoteEvent> {
    let r = root as i32;
    let vels = [115, 76, 92, 78, 88];
    let offsets = [0.0, 1.5, 2.0, 2.5, 3.5];
    let pitches = [r, r + 7, r, r + 7, r + 12];
    let durs = [1.2, 0.8, 0.5, 0.5, 0.4];
    offsets.iter().zip(pitches.iter()).zip(durs.iter()).zip(vels.iter())
        .map(|(((o, p), d), v)| {
            NoteEvent::new((*p).clamp(0, 127) as u8, *d, *v, bar_start + o)
        }).collect()
}

// ─── COMPING (CHORD) PATTERNS ─────────────────────────────

/// Bossa guitar comping - syncopated chord stabs
pub fn comping_bossa_guitar(root: u8, chord: &ChordSymbol, bar_start: f32) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        let degs = &ct.degrees;
        // Bossa rhythm: [1, 1+, 2+, 3, 3+, 4, 4+]
        let pattern = [(0.0, 0.6, 0, 105), (1.5, 0.5, 1, 80),
                       (2.5, 0.5, 2, 85), (3.0, 0.6, 0, 98),
                       (3.5, 0.4, 1, 78)];
        let root = root as i32 + 12; // one octave up
        return pattern.iter().map(|&(pos, dur, deg_idx, vel)| {
            let d = degs[deg_idx % degs.len()];
            let pitch = (root + d.pitch() as i32).clamp(0, 127) as u8;
            NoteEvent::new(pitch, dur, vel, bar_start + pos)
        }).collect();
    }
    Vec::new()
}

/// Piano comping - sparser pattern
pub fn comping_piano(root: u8, chord: &ChordSymbol, bar_start: f32) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        let degs = &ct.degrees;
        let pattern = [(0.0, 1.0, 0, 100), (2.0, 1.2, 1, 88), (3.0, 0.8, 2, 82)];
        let root = root as i32 + 12;
        return pattern.iter().map(|&(pos, dur, deg_idx, vel)| {
            let d = degs[deg_idx % degs.len()];
            let pitch = (root + d.pitch() as i32).clamp(0, 127) as u8;
            NoteEvent::new(pitch, dur, vel, bar_start + pos)
        }).collect();
    }
    Vec::new()
}

// ─── PAD PATTERNS ─────────────────────────────────────────

/// Sustained string pad
pub fn pad_sustained(root: u8, chord: &ChordSymbol, bar_start: f32, duration: f32) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        let vels = [80, 68, 72];
        return ct.degrees.iter().take(3).enumerate().map(|(i, &d)| {
            let pitch = (root as i32 + 12 + d.pitch() as i32).clamp(0, 127) as u8;
            NoteEvent::new(pitch, duration - 0.05, vels[i], bar_start)
        }).collect();
    }
    Vec::new()
}

// ─── MELODY PATTERNS ──────────────────────────────────────

/// Call and response melodic phrase
pub fn melody_call_response(root: u8, chord: &ChordSymbol, bar_start: f32, bar_idx: usize) -> Vec<NoteEvent> {
    if let Some(ct) = chord.chord_type() {
        let degs = &ct.degrees;
        let root = root as i32 + 12;
        // Alternate between two patterns for musical interest
        let pattern = if bar_idx.is_multiple_of(2) {
            vec![(0.0, 0.6, 0, 108), (1.0, 0.5, 2, 82),
                 (2.0, 0.4, 1, 90), (3.0, 0.8, 0, 100)]
        } else {
            vec![(0.5, 0.5, 2, 85), (1.5, 0.4, 1, 78),
                 (2.5, 0.5, 0, 92), (3.5, 0.6, 2, 80)]
        };
        return pattern.iter().map(|&(pos, dur, deg_idx, vel)| {
            let d = degs[deg_idx % degs.len()];
            let pitch = (root + d.pitch() as i32).clamp(0, 127) as u8;
            NoteEvent::new(pitch, dur, vel, bar_start + pos)
        }).collect();
    }
    Vec::new()
}
