use crate::config::OrbSize;
use crate::geometry::{push_dot, smoothstep};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_icon};
use crate::state::OrbState;

const HOLD: f64 = 1.4;
const MORPH: f64 = 0.9;
const SEGMENT: f64 = HOLD + MORPH;
const SHAPES: usize = 3;

struct Polygon {
    vertices: Vec<(f64, f64)>,
    lengths: Vec<f64>,
    total: f64,
}

impl Polygon {
    fn new(vertices: &[(f64, f64)]) -> Self {
        let mut lengths = Vec::with_capacity(vertices.len());
        let mut total = 0.0;
        for index in 0..vertices.len() {
            let a = vertices[index];
            let b = vertices[(index + 1) % vertices.len()];
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            lengths.push(length);
            total += length;
        }
        Self {
            vertices: vertices.to_vec(),
            lengths,
            total,
        }
    }

    fn point(&self, fraction: f64) -> (f64, f64) {
        let mut target = fraction * self.total;
        let mut index = 0;
        while target > self.lengths[index] && index < self.vertices.len() - 1 {
            target -= self.lengths[index];
            index += 1;
        }
        let a = self.vertices[index];
        let b = self.vertices[(index + 1) % self.vertices.len()];
        let local = if self.lengths[index] > 0.0 {
            (target / self.lengths[index]).min(1.0)
        } else {
            0.0
        };
        (a.0 + (b.0 - a.0) * local, a.1 + (b.1 - a.1) * local)
    }
}

enum Shape {
    Circle,
    Polygon(Polygon),
}

impl Shape {
    fn point(&self, fraction: f64) -> (f64, f64) {
        match self {
            Self::Circle => {
                let angle = -std::f64::consts::FRAC_PI_2 + fraction * std::f64::consts::TAU;
                (angle.cos() * 0.24, angle.sin() * 0.24)
            }
            Self::Polygon(polygon) => polygon.point(fraction),
        }
    }
}

/// A dotted outline morphing circle → triangle → square — the reference
/// "shaping" mode. Each shape holds for 1.4s, then morphs for 0.9s.
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Shaping, size);
    let circle = Shape::Circle;
    let triangle = Shape::Polygon(Polygon::new(&[(0.0, -0.26), (0.24, 0.16), (-0.24, 0.16)]));
    let square = Shape::Polygon(Polygon::new(&[
        (0.0, -0.2),
        (0.2, -0.2),
        (0.2, 0.2),
        (-0.2, 0.2),
        (-0.2, -0.2),
    ]));
    let shapes = [&circle, &triangle, &square];

    let local_cycle = time.rem_euclid(SEGMENT * SHAPES as f64);
    let shape_index = (local_cycle / SEGMENT).floor() as usize;
    let local = local_cycle - shape_index as f64 * SEGMENT;
    let morph = if local > HOLD {
        smoothstep(0.0, 1.0, ((local - HOLD) / MORPH) as f32)
    } else {
        0.0
    };

    let spread = f64::from(preset.spread);
    let first = shapes[shape_index];
    let second = shapes[(shape_index + 1) % SHAPES];

    const PATH_SAMPLES: usize = 160;
    let mut points = Vec::with_capacity(PATH_SAMPLES);
    for index in 0..PATH_SAMPLES {
        let fraction = index as f64 / PATH_SAMPLES as f64;
        let a = first.point(fraction);
        let b = second.point(fraction);
        let blend = f64::from(morph);
        points.push((
            (a.0 + (b.0 - a.0) * blend) * spread,
            (a.1 + (b.1 - a.1) * blend) * spread,
        ));
    }

    let mut lengths = Vec::with_capacity(PATH_SAMPLES);
    let mut total = 0.0;
    for index in 0..PATH_SAMPLES {
        let a = points[index];
        let b = points[(index + 1) % PATH_SAMPLES];
        let length = (b.0 - a.0).hypot(b.1 - a.1);
        lengths.push(length);
        total += length;
    }

    let icon_density = scaled_icon(1.0, preset.count);
    let count = ((34.0 * icon_density).round() as usize).max(6);
    let dot_radius = 0.021 * preset.radius_mul * 1.35 * spread as f32;
    let dot_radius = (dot_radius * preset.reference_size).max(0.35);
    let pulse = 1.0 + 0.02 * (local * 3.1).sin();

    let mut segment = 0;
    let mut accumulated = 0.0;
    for index in 0..count {
        let target = (index as f64 / count as f64) * total;
        while accumulated + lengths[segment] < target && segment < PATH_SAMPLES - 1 {
            accumulated += lengths[segment];
            segment += 1;
        }
        let a = points[segment];
        let b = points[(segment + 1) % PATH_SAMPLES];
        let local_fraction = if lengths[segment] > 0.0 {
            ((target - accumulated) / lengths[segment]).min(1.0)
        } else {
            0.0
        };
        let x = (a.0 + (b.0 - a.0) * local_fraction) * pulse;
        let y = (a.1 + (b.1 - a.1) * local_fraction) * pulse;
        push_dot(
            out,
            (x * 2.0) as f32,
            (y * 2.0) as f32,
            0.0,
            dot_radius,
            0.1,
            1.0,
        );
    }
}
