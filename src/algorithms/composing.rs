use crate::config::OrbSize;
use crate::geometry::{Projector, fib_dir, push_dot};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_count, scaled_pair};
use crate::state::OrbState;

/// An undulating multi-lane sash — the reference "composing" mode (ribbon).
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    sample_ribbon(OrbState::Composing, time, size, out);
}

/// The face-on ring variant used by "breathing".
pub(crate) fn sample_face_on(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    sample_ribbon(OrbState::Breathing, time, size, out);
}

fn sample_ribbon(state: OrbState, time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(state, size);
    let face_on = state == OrbState::Breathing;
    let radius = 0.78;
    let spin = f64::from(preset.spin);
    let camera_tilt = 0.3;
    let projector = Projector::new(time * 0.1 * spin, camera_tilt);
    // The reference ring profile sets ghostN to zero; only the ribbon has a
    // ghost sphere behind the strands.
    let ghost_count = if face_on {
        0
    } else {
        scaled_count(150.0, preset.count)
    };
    let ghost_radius = preset.fixed_radius(0.8);
    let base_radius = preset.radius(1.1);
    let depth_radius = preset.radius(1.7);

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

    let ya = time * 0.24 * spin;
    let ta = if face_on {
        -camera_tilt
    } else {
        0.55 + 0.3 * (time * 0.18).sin() * spin
    };
    let ux = ya.cos();
    let uy = 0.0;
    let uz = ya.sin();
    let vx = -uz * ta.sin();
    let vy = ta.cos();
    let vz = ux * ta.sin();
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;

    let wobble_amplitude = 0.23 * f64::from(preset.wob_mul);
    let base_r = if face_on {
        f64::from(radius) / (1.0 + 0.85 * wobble_amplitude)
    } else {
        f64::from(radius)
    };

    let (base_lanes, segs) = scaled_pair(5.0, 88.0, preset.count);
    let lanes = ((base_lanes as f32 * preset.band_mul).round() as usize).max(1);

    for lane in 0..lanes {
        let lane_offset = (lane as f64 - (lanes - 1) as f64 * 0.5) * 0.075;
        let edge =
            (lane as f32 - (lanes - 1) as f32 * 0.5).abs() / ((lanes - 1) as f32 * 0.5).max(1.0);

        for segment in 0..segs {
            let angle = (segment as f64 / segs as f64) * std::f64::consts::TAU;
            let wobble = (0.16 * (angle * 3.0 - time * 1.7 + lane as f64 * 0.22).sin()
                + 0.07 * (angle * 5.0 + time * 1.1).sin())
                * f64::from(preset.wob_mul);
            let radial = if face_on { 1.0 + wobble } else { 1.0 };
            let offset = if face_on {
                lane_offset
            } else {
                lane_offset + wobble
            };

            let x = ux * angle.cos() + vx * angle.sin() + nx * offset;
            let y = uy * angle.cos() + vy * angle.sin() + ny * offset;
            let z = uz * angle.cos() + vz * angle.sin() + nz * offset;
            let length = (x * x + y * y + z * z).max(1.0e-6).sqrt();
            let ring_radius = base_r * radial;
            let (px, py, pz) = projector.project(
                (x / length * ring_radius) as f32,
                (y / length * ring_radius) as f32,
                (z / length * ring_radius) as f32,
            );
            let depth = (pz / radius + 1.0) * 0.5;
            let dot_radius = (base_radius + depth_radius * depth) * (1.0 - 0.25 * edge);
            let white = 0.52 - 0.44 * depth + 0.18 * edge;
            let alpha = 0.4 + 0.6 * depth;
            push_dot(out, px, py, pz / radius, dot_radius, white, alpha);
        }
    }
}
