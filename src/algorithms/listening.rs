use crate::config::OrbSize;
use crate::geometry::{Projector, push_dot};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_pair};
use crate::state::OrbState;

/// A waveform rolling through latitude rings — the reference "listening" mode.
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Listening, size);
    let (rings, lon_density) = scaled_pair(15.0, 40.0, preset.count);
    let radius = 0.874;
    let projector = Projector::new(time * 0.18, 0.38);
    let base_radius = preset.radius(0.6);
    let depth_radius = preset.radius(1.7);

    for ring in 0..=rings {
        let lat =
            -std::f64::consts::FRAC_PI_2 + (ring as f64 / rings as f64) * std::f64::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let wave = 0.62 * (time * 2.1 - ring as f64 * 0.52).sin()
            + 0.38 * (time * 1.27 + ring as f64 * 0.83).sin();
        let ring_radius = radius * (0.88 + 0.105 * wave);
        let lon_count = ((cos_lat.abs() * lon_density as f64).round() as usize).max(1);

        for lon_index in 0..lon_count {
            let lon = (lon_index as f64 / lon_count as f64) * std::f64::consts::TAU;
            let x = cos_lat * lon.cos() * ring_radius;
            let y = sin_lat * ring_radius;
            let z = cos_lat * lon.sin() * ring_radius;
            let (px, py, pz) = projector.project(x as f32, y as f32, z as f32);
            let depth = (f64::from(pz) / radius + 1.0) * 0.5;
            let crest = wave.max(0.0) as f32;
            let dot_radius = (base_radius + depth_radius * depth as f32) * (1.0 + 0.4 * crest);
            let white = 0.66 - 0.56 * depth as f32 - 0.1 * crest;
            push_dot(
                out,
                px,
                py,
                (f64::from(pz) / radius) as f32,
                dot_radius,
                white,
                1.0,
            );
        }
    }
}
