use crate::config::OrbSize;
use crate::geometry::{Projector, Vec3, hash_d, push_dot};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_count};
use crate::state::OrbState;

/// Particles on tilted orbits — the reference "working" mode.
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Working, size);
    let orbit_count = scaled_count(12.0, preset.count);
    let ghost_count = scaled_count(40.0, preset.count);
    let particle_count = 3;
    let ghost_radius = preset.radius(0.9);
    let particle_radius = preset.radius(1.2);
    let particle_radius_depth = preset.radius(1.6);
    let projector = Projector::new(time * 0.12, 0.3);
    let radius = 0.82;

    for orbit in 0..orbit_count {
        let h1 = hash_d(orbit as f64, 1.7);
        let h2 = hash_d(orbit as f64, 5.2);
        let h3 = hash_d(orbit as f64, 8.9);
        let orbit_radius = radius * (0.45 + 0.52 * h1);
        let theta = f64::from(h1) * std::f64::consts::TAU;
        let phi = (2.0 * f64::from(h2) - 1.0).clamp(-1.0, 1.0).acos();

        let normal = Vec3::new(
            (phi.sin() * theta.cos()) as f32,
            phi.cos() as f32,
            (phi.sin() * theta.sin()) as f32,
        );
        let mut u = Vec3::new(-normal.y, normal.x, 0.0);
        let u_length = (u.x * u.x + u.y * u.y).max(1.0e-6).sqrt();
        u = Vec3::new(u.x / u_length, u.y / u_length, 0.0);
        let v = Vec3::new(
            normal.y * u.z - normal.z * u.y,
            normal.z * u.x - normal.x * u.z,
            normal.x * u.y - normal.y * u.x,
        );
        let speed = (0.25 + 0.55 * f64::from(h3)) * if h3 > 0.5 { 1.0 } else { -1.0 };

        for index in 0..ghost_count {
            let angle = (index as f64 / ghost_count as f64) * std::f64::consts::TAU;
            let point = orbit_point(u, v, orbit_radius, angle);
            let (x, y, z) = projector.project(point.x, point.y, point.z);
            let depth = (z / orbit_radius + 1.0) * 0.5;
            let alpha = 0.5 * (0.4 + 0.6 * depth);
            push_dot(out, x, y, z / orbit_radius, ghost_radius, 0.72, alpha);
        }

        for index in 0..particle_count {
            let angle = time * speed
                + (index as f64 / particle_count as f64) * std::f64::consts::TAU
                + f64::from(h2) * 6.0;
            let point = orbit_point(u, v, orbit_radius, angle);
            let (x, y, z) = projector.project(point.x, point.y, point.z);
            let depth = (z / orbit_radius + 1.0) * 0.5;
            let dot_radius = particle_radius + particle_radius_depth * depth;
            let white = 0.3 - 0.22 * depth;
            push_dot(out, x, y, z / orbit_radius, dot_radius, white, 1.0);
        }
    }
}

fn orbit_point(u: Vec3, v: Vec3, radius: f32, angle: f64) -> Vec3 {
    let (sin, cos) = (angle.sin() as f32, angle.cos() as f32);
    Vec3::new(
        (u.x * cos + v.x * sin) * radius,
        (u.y * cos + v.y * sin) * radius,
        (u.z * cos + v.z * sin) * radius,
    )
}
