use crate::config::OrbSize;
use crate::geometry::{Projector, fib_dir, fract, push_dot};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_count};
use crate::state::OrbState;

/// Three strands plait around a ghost sphere — the reference "weaving" mode
/// (braid).
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Weaving, size);
    let ghost_count = scaled_count(150.0, preset.count);
    let strand_count = scaled_count(52.0, preset.count);
    let radius = 0.76;
    let projector = Projector::new(time * 0.4, 0.3);
    let ghost_radius = preset.fixed_radius(0.8);
    let base_radius = preset.radius(1.2);
    let depth_radius = preset.radius(1.8);

    for index in 0..ghost_count {
        let direction = fib_dir(index, ghost_count);
        let (x, y, z) = projector.project(
            direction.x * radius,
            direction.y * radius,
            direction.z * radius,
        );
        let depth = (z / radius + 1.0) * 0.5;
        let alpha = 0.1 + 0.22 * depth;
        push_dot(out, x, y, z / radius, ghost_radius, 0.78, alpha);
    }

    for strand in 0..3 {
        let phase = (strand as f64 / 3.0) * std::f64::consts::TAU;
        for index in 0..strand_count {
            let u = (fract(index as f64 / strand_count as f64 + time * 0.045) * 2.0 - 1.0) * 0.96;
            let surface = (1.0 - u * u).max(0.0).sqrt();
            let end_fade = ((1.0 - u.abs()) / 0.1).min(1.0);
            let angle = u * std::f64::consts::PI * 3.0 + phase;
            let weave =
                1.0 + 0.075 * (u * std::f64::consts::PI * 6.0 + phase * 2.0 + time * 0.8).sin();
            let ring_radius = surface * f64::from(radius) * weave;
            let x = angle.cos() * ring_radius;
            let y = u * f64::from(radius) * weave;
            let z = angle.sin() * ring_radius;
            let (px, py, pz) = projector.project(x as f32, y as f32, z as f32);
            let depth = (pz / radius + 1.0) * 0.5;
            let dot_radius = base_radius + depth_radius * depth;
            let white = 0.55 - 0.45 * depth;
            let alpha = end_fade * (0.45 + 0.55 * f64::from(depth));
            push_dot(out, px, py, pz / radius, dot_radius, white, alpha as f32);
        }
    }
}
