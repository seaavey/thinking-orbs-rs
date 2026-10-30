use crate::config::OrbSize;
use crate::state::OrbState;

/// Per-state, per-size tuning values from the reference preset table.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Preset {
    pub(crate) speed: f64,
    pub(crate) count: f32,
    pub(crate) radius_mul: f32,
    pub(crate) radius_scale: f32,
    pub(crate) reference_size: f32,
    pub(crate) scan_mul: f32,
    pub(crate) dim_base: f32,
    pub(crate) spin: f32,
    pub(crate) band_mul: f32,
    pub(crate) wob_mul: f32,
    pub(crate) spread: f32,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            speed: 1.0,
            count: 1.0,
            radius_mul: 1.0,
            radius_scale: 1.0,
            reference_size: 64.0,
            scan_mul: 1.0,
            dim_base: 1.0,
            spin: 1.0,
            band_mul: 1.0,
            wob_mul: 1.0,
            spread: 1.0,
        }
    }
}

pub(crate) fn for_state(state: OrbState, size: OrbSize) -> Preset {
    let mut preset = Preset::default();
    let compact = size == OrbSize::Compact;
    preset.reference_size = if compact { 20.0 } else { 64.0 };
    preset.radius_scale = (preset.reference_size / 300.0).powf(0.6);

    match state {
        OrbState::Idle => {}
        OrbState::Working => {
            preset.speed = if compact { 3.9 } else { 1.885 };
            preset.count = if compact { 0.238 } else { 1.0 };
            preset.radius_mul = if compact { 2.4 } else { 1.0 };
        }
        OrbState::Searching => {
            preset.speed = if compact { 2.665 } else { 2.015 };
            preset.count = if compact { 0.105 } else { 0.42 };
            preset.radius_mul = if compact { 1.75 } else { 1.15 };
            preset.scan_mul = if compact { 4.335 } else { 4.08 };
            preset.dim_base = 0.45;
        }
        OrbState::Solving => {
            preset.speed = if compact { 1.95 } else { 1.82 };
            preset.count = if compact { 0.088 } else { 0.35 };
            preset.radius_mul = if compact { 1.9 } else { 1.05 };
        }
        OrbState::Listening => {
            preset.speed = if compact { 3.998 } else { 4.388 };
            preset.count = if compact { 0.105 } else { 0.341 };
            preset.radius_mul = if compact { 1.6 } else { 1.0 };
        }
        OrbState::Connecting => {
            preset.speed = if compact { 6.63 } else { 3.315 };
            preset.count = if compact { 0.25 } else { 1.35 };
            preset.radius_mul = if compact { 1.52 } else { 0.95 };
        }
        OrbState::Weaving => {
            preset.speed = if compact { 2.75 } else { 1.625 };
            preset.count = if compact { 0.1125 } else { 0.5 };
            preset.radius_mul = if compact { 1.36 } else { 1.0 };
        }
        OrbState::Composing => {
            preset.speed = if compact { 3.12 } else { 2.34 };
            preset.count = if compact { 0.051 } else { 0.25 };
            preset.radius_mul = if compact { 1.073 } else { 0.85 };
            preset.spin = 0.0;
            preset.band_mul = if compact { 4.94 } else { 3.9 };
        }
        OrbState::Breathing => {
            preset.speed = if compact { 3.78 } else { 3.24 };
            preset.count = if compact { 0.028 } else { 0.25 };
            preset.radius_mul = if compact { 1.622 } else { 0.956 };
            preset.spin = 0.0;
            preset.band_mul = if compact { 3.968 } else { 3.627 };
            preset.wob_mul = if compact { 0.565 } else { 0.368 };
        }
        OrbState::Shaping => {
            preset.speed = if compact { 2.08 } else { 2.405 };
            preset.count = if compact { 0.53 } else { 0.702 };
            preset.radius_mul = if compact { 1.011 } else { 0.395 };
            preset.spread = 1.45;
        }
    }

    preset
}

pub(crate) fn scaled_count(base: f32, scale: f32) -> usize {
    (base * scale).round().max(1.0) as usize
}

pub(crate) fn scaled_pair(first: f32, second: f32, scale: f32) -> (usize, usize) {
    let root = scale.max(0.0).sqrt();
    (
        (first * root).round().max(2.0) as usize,
        (second * root).round().max(2.0) as usize,
    )
}

pub(crate) fn scaled_icon(base: f32, scale: f32) -> f32 {
    (base * scale).max(0.02)
}

impl Preset {
    pub(crate) fn radius(&self, base: f32) -> f32 {
        base * self.radius_mul * self.radius_scale
    }

    /// Radius for dots the reference scales with the canvas only. Its ghost
    /// lattices use a bare `0.8 * radiusScale(size)` and ignore the per-state
    /// radius multiplier.
    pub(crate) fn fixed_radius(&self, base: f32) -> f32 {
        base * self.radius_scale
    }
}
