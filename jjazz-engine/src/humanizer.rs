use rand::Rng;
use crate::phrase::Phrase;

/// Humanize notes by adding random timing and velocity deviations.
pub struct Humanizer {
    config: HumanizerConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct HumanizerConfig {
    pub timing_randomness: f32,
    pub timing_bias: f32,
    pub velocity_randomness: f32,
}

impl Default for HumanizerConfig {
    fn default() -> Self { Self { timing_randomness: 0.2, timing_bias: 0.0, velocity_randomness: 0.2 } }
}

pub const MAX_TIMING_DEV: f32 = 0.2;
pub const MAX_VEL_DEV: f32 = 30.0;

impl Humanizer {
    pub fn new(config: HumanizerConfig) -> Self { Self { config } }

    pub fn humanize(&self, phrase: &mut Phrase) {
        let mut rng = rand::thread_rng();
        for ne in phrase.notes.iter_mut() {
            let tf: f32 = rng.gen_range(-1.0..1.0);
            let vf: f32 = rng.gen_range(-1.0..1.0);

            let pos_shift = tf * MAX_TIMING_DEV * self.config.timing_randomness
                + 0.2 * self.config.timing_bias;
            ne.position = (ne.position + pos_shift).max(0.0);

            let vel_shift = (vf * MAX_VEL_DEV * self.config.velocity_randomness) as i32;
            ne.velocity = ((ne.velocity as i32 + vel_shift).clamp(0, 127)) as u8;
        }
        phrase.sort();
    }
}
