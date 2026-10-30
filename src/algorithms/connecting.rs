use crate::config::OrbSize;
use crate::geometry::{Projector, Vec3, fib_dir, fract, hash_d, push_dot, vnoise};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_count};
use crate::state::OrbState;

/// A constellation that wires itself, with packets running the edges — the
/// reference "connecting" mode (web).
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Connecting, size);
    let node_count = scaled_count(30.0, preset.count);
    let signal_count = scaled_count(5.0, preset.count);
    let threshold = 0.72_f32;
    let radius = 0.8;
    let projector = Projector::new(time * 0.12, 0.32);
    let node_radius = preset.radius(1.4);
    let node_radius_depth = preset.radius(1.8);

    let mut nodes = Vec::with_capacity(node_count);
    for index in 0..node_count {
        let base = fib_dir(index, node_count);
        let wander =
            |seed: f64, speed: f64| (f64::from(vnoise(seed, time * speed)) - 0.5) * 2.0 * 0.3;
        let x = f64::from(base.x) + wander(index as f64 * 0.31 + 9.0, 0.24);
        let y = f64::from(base.y) + wander(index as f64 * 0.53 + 27.0, 0.21);
        let z = f64::from(base.z) + wander(index as f64 * 0.77 + 55.0, 0.27);
        let length = (x * x + y * y + z * z).max(1.0e-6).sqrt();
        nodes.push(Vec3::new(
            (x / length) as f32,
            (y / length) as f32,
            (z / length) as f32,
        ));
    }

    // Lines are sampled as faint dots because the terminal frame is a dot
    // field. They are deliberately low-contrast so nodes sit on top.
    let (width, height) = size.dimensions();
    for left in 0..nodes.len() {
        for right in (left + 1)..nodes.len() {
            let delta = nodes[left] - nodes[right];
            let distance = delta.length();
            if distance >= threshold {
                continue;
            }
            let first = project_node(nodes[left], radius, projector);
            let second = project_node(nodes[right], radius, projector);
            let depth = (first.1 + second.1) * 0.5;
            let alpha = (1.0 - distance / threshold) * (0.3 + 0.55 * depth);
            let span_x = (second.0.x - first.0.x).abs() * f32::from(width) * 0.5;
            let span_y = (second.0.y - first.0.y).abs() * f32::from(height) * 0.5;
            let steps = (span_x.max(span_y).ceil() as usize).clamp(2, 24);
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let point = first.0.lerp(second.0, t);
                push_dot(out, point.x, point.y, point.z / radius, 0.35, 0.42, alpha);
            }
        }
    }

    for (index, node) in nodes.iter().copied().enumerate() {
        let (point, depth) = project_node(node, radius, projector);
        let pulse = 1.0 + 0.25 * (time * 1.4 + index as f64 * 2.7).sin() as f32;
        let dot_radius = (node_radius + node_radius_depth * depth) * pulse;
        let white = 0.55 - 0.45 * depth;
        push_dot(
            out,
            point.x,
            point.y,
            point.z / radius,
            dot_radius,
            white,
            1.0,
        );
    }

    for signal in 0..signal_count {
        let segment = (time * 0.55 + signal as f64 * 7.31).floor();
        let first = (hash_d(segment, signal as f64 * 3.1 + 1.7) * node_count as f32)
            .floor()
            .min((node_count - 1) as f32) as usize;
        let second = (hash_d(segment, signal as f64 * 5.7 + 4.2) * node_count as f32)
            .floor()
            .min((node_count - 1) as f32) as usize;
        if first == second {
            continue;
        }
        let fraction = fract(time * 0.55 + signal as f64 * 7.31) as f32;
        let point = nodes[first].lerp(nodes[second], fraction).normalized();
        let (projected, depth) = project_node(point, radius, projector);
        let dot_radius = node_radius * 1.5 + node_radius_depth * depth;
        let alpha = 0.5 + 0.5 * depth;
        push_dot(
            out,
            projected.x,
            projected.y,
            projected.z / radius,
            dot_radius,
            0.05,
            alpha,
        );
    }
}

fn project_node(node: Vec3, radius: f32, projector: Projector) -> (Vec3, f32) {
    let (x, y, z) = projector.project(node.x * radius, node.y * radius, node.z * radius);
    (Vec3::new(x, y, z), (z / radius + 1.0) * 0.5)
}
