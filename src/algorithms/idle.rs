use std::f64::consts::TAU;

use crate::config::OrbSize;
use crate::geometry::{Projector, circle_point, loop_angle, push_dot};
use crate::particle::Particle;

/// A quiet, slowly drifting pair of rings. Idle is the terminal-only resting
/// state and is intentionally calmer than the nine reference modes.
pub(crate) fn sample(time: f64, _size: OrbSize, out: &mut Vec<Particle>) {
    const RINGS: [(f32, usize, f64); 2] = [(0.52, 64, 0.0), (0.82, 80, 2.0)];

    for (ring, (base, count, phase)) in RINGS.into_iter().enumerate() {
        let breath = (time * 0.45 + phase).sin() as f32;
        let radius = base * (1.0 + 0.025 * breath);
        let projector = Projector::new(time * 0.055 + phase * 0.05, 0.42 + ring as f64 * 0.08);

        for index in 0..count {
            let angle = loop_angle(index, count);
            let point = circle_point(radius, angle);
            let (x, y, z) = projector.project(point.x, point.y, point.z);
            let depth = (z / radius + 1.0) * 0.5;
            let intensity_alpha = 0.16 + 0.30 * depth + 0.08 * (breath * 0.5 + 0.5);
            push_dot(
                out,
                x,
                y,
                z / radius,
                0.35 + 0.15 * depth,
                0.72,
                intensity_alpha,
            );
        }
    }

    for index in 0..6 {
        let angle = TAU * f64::from(index) / 6.0 + time * 0.03;
        let point = circle_point(0.035, angle);
        let projector = Projector::new(time * 0.03, 0.3);
        let (x, y, z) = projector.project(point.x, point.y, point.z);
        push_dot(out, x, y, z, 0.4, 0.78, 0.22);
    }
}
