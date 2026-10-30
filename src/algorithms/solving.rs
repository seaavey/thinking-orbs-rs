use crate::config::OrbSize;
use crate::geometry::{Projector, Vec3, hash_d, push_dot, rotate_axis};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_pair};
use crate::state::OrbState;

const MOVE_COUNT: usize = 14;
const SLOT_DURATION: f64 = 0.42;
const REST_DURATION: f64 = 1.2;

#[derive(Clone, Copy)]
struct Move {
    axis: u8,
    lo: f64,
    hi: f64,
    angle: f64,
}

struct SolveCycle {
    amount: Vec<f64>,
    active: Option<usize>,
}

/// Bands twist in quarter turns, scramble, then replay in reverse — the
/// reference "solving" mode (rubik).
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Solving, size);
    let (lat_rings, lon_density) = scaled_pair(15.0, 40.0, preset.count);
    let radius = 0.82;
    let tilt = 0.35 + 0.1 * (time * 0.9).sin();
    let projector = Projector::new(time * 0.55, tilt);
    let moves = make_moves();
    let cycle = solve_cycle(time);
    let base_radius = preset.radius(0.6);
    let depth_radius = preset.radius(1.7);
    let active_radius = preset.radius(0.3);

    for lat_index in 0..=lat_rings {
        let lat = -std::f64::consts::FRAC_PI_2
            + (lat_index as f64 / lat_rings as f64) * std::f64::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = ((cos_lat.abs() * lon_density as f64).round() as usize).max(1);

        for lon_index in 0..lon_count {
            let lon = (lon_index as f64 / lon_count as f64) * std::f64::consts::TAU;
            let point = Vec3::new(
                (cos_lat * lon.cos()) as f32,
                sin_lat as f32,
                (cos_lat * lon.sin()) as f32,
            );
            let (point, in_active) = apply_moves(point, &moves, &cycle);
            let (x, y, z) = projector.project(point.x * radius, point.y * radius, point.z * radius);
            let depth = (z / radius + 1.0) * 0.5;
            let dot_radius =
                base_radius + depth_radius * depth + if in_active { active_radius } else { 0.0 };
            let white = 0.62 - 0.54 * depth - if in_active { 0.14 } else { 0.0 };
            push_dot(out, x, y, z / radius, dot_radius, white, 1.0);
        }
    }
}

fn make_moves() -> Vec<Move> {
    let mut moves = Vec::with_capacity(MOVE_COUNT);
    for index in 0..MOVE_COUNT {
        let index_f = index as f64;
        let axis = (f64::from(hash_d(index_f, 2.3)) * 3.0).floor().min(2.0) as u8;
        let lo = -1.0 + 0.5 * (f64::from(hash_d(index_f, 5.9)) * 4.0).floor().min(3.0);
        let direction = if hash_d(index_f, 7.7) < 0.5 {
            1.0
        } else {
            -1.0
        };
        moves.push(Move {
            axis,
            lo,
            hi: lo + 0.5,
            angle: direction * std::f64::consts::FRAC_PI_2,
        });
    }
    moves
}

fn solve_cycle(time: f64) -> SolveCycle {
    let total = 2.0 * MOVE_COUNT as f64 * SLOT_DURATION + REST_DURATION;
    let local = time.rem_euclid(total);
    let mut amount = vec![0.0; MOVE_COUNT];
    let mut active = None;

    if local < 2.0 * MOVE_COUNT as f64 * SLOT_DURATION {
        let slot = (local / SLOT_DURATION).floor() as usize;
        let progress = (local - slot as f64 * SLOT_DURATION) / SLOT_DURATION;
        let clamped = (progress / 0.7).min(1.0);
        let eased = 1.0 - (1.0 - clamped).powi(3);

        if slot < MOVE_COUNT {
            for value in amount.iter_mut().take(slot) {
                *value = 1.0;
            }
            amount[slot] = eased;
            active = Some(slot);
        } else {
            let index = 2 * MOVE_COUNT - 1 - slot;
            for value in amount.iter_mut().take(index) {
                *value = 1.0;
            }
            amount[index] = 1.0 - eased;
            active = Some(index);
        }
    }

    SolveCycle { amount, active }
}

fn apply_moves(point: Vec3, moves: &[Move], cycle: &SolveCycle) -> (Vec3, bool) {
    let mut point = point;
    let mut in_active = false;

    for (index, movement) in moves.iter().enumerate() {
        if cycle.amount[index] <= 0.0 {
            continue;
        }
        let coordinate = match movement.axis {
            0 => f64::from(point.x),
            1 => f64::from(point.y),
            _ => f64::from(point.z),
        };
        if coordinate < movement.lo || coordinate >= movement.hi {
            continue;
        }
        if cycle.active == Some(index) {
            in_active = true;
        }
        point = rotate_axis(point, movement.axis, movement.angle * cycle.amount[index]);
    }

    (point, in_active)
}
