use std::ops::{Add, Sub};

use crate::particle::Particle;

/// A point in the animation's local three-dimensional space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vec3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

impl Vec3 {
    pub(crate) const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub(crate) const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub(crate) fn normalized(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            Self::ZERO
        } else {
            Self::new(self.x / length, self.y / length, self.z / length)
        }
    }

    pub(crate) fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            lerp(self.x, other.x, t),
            lerp(self.y, other.y, t),
            lerp(self.z, other.z, t),
        )
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// Linear interpolation.
pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Positive fractional part of a value.
pub(crate) fn fract(value: f64) -> f64 {
    value - value.floor()
}

/// A smooth Hermite interpolation between two edges.
pub(crate) fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    if (edge1 - edge0).abs() <= f32::EPSILON {
        return if value < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The reference implementation's deterministic hash in [0, 1).
pub(crate) fn hash_d(a: f64, b: f64) -> f32 {
    let h = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    (h - h.floor()) as f32
}

/// Value noise on a 2D lattice, matching the reference web mode.
pub(crate) fn vnoise(x: f64, y: f64) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let mut fx = x - xi;
    let mut fy = y - yi;
    fx = fx * fx * (3.0 - 2.0 * fx);
    fy = fy * fy * (3.0 - 2.0 * fy);
    let a = f64::from(hash_d(xi, yi));
    let b = f64::from(hash_d(xi + 1.0, yi));
    let c = f64::from(hash_d(xi, yi + 1.0));
    let d = f64::from(hash_d(xi + 1.0, yi + 1.0));
    (a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy) as f32
}

/// A stable direction on a unit sphere using the Fibonacci lattice.
pub(crate) fn fib_dir(index: usize, count: usize) -> Vec3 {
    if count == 0 {
        return Vec3::ZERO;
    }
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let y = 1.0 - (2.0 * (index as f64 + 0.5)) / count as f64;
    let radius = (1.0 - y * y).max(0.0).sqrt();
    let angle = index as f64 * golden;
    Vec3::new(
        (radius * angle.cos()) as f32,
        y as f32,
        (radius * angle.sin()) as f32,
    )
}

/// Shortest signed angular distance, wrapped to (-pi, pi].
pub(crate) fn angle_delta_signed(a: f64, b: f64) -> f64 {
    (a - b).sin().atan2((a - b).cos())
}

/// Shared spin + tilt + orthographic projection from the reference engine.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Projector {
    yaw_sin: f64,
    yaw_cos: f64,
    tilt_sin: f64,
    tilt_cos: f64,
}

impl Projector {
    pub(crate) fn new(yaw: f64, tilt: f64) -> Self {
        Self {
            yaw_sin: yaw.sin(),
            yaw_cos: yaw.cos(),
            tilt_sin: tilt.sin(),
            tilt_cos: tilt.cos(),
        }
    }

    /// Projects a point into a centered normalized canvas. Positive y is down,
    /// matching terminal row direction. Depth is the rotated z coordinate.
    pub(crate) fn project(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let x = f64::from(x);
        let y = f64::from(y);
        let z = f64::from(z);
        let x1 = x * self.yaw_cos + z * self.yaw_sin;
        let z1 = -x * self.yaw_sin + z * self.yaw_cos;
        let y1 = y * self.tilt_cos - z1 * self.tilt_sin;
        let z2 = y * self.tilt_sin + z1 * self.tilt_cos;
        (x1 as f32, -y1 as f32, z2 as f32)
    }
}

/// Converts reference dot values into a particle. The reference ink value is
/// 0 for the darkest ink; terminal brightness is its inverse, weighted by the
/// dot's alpha. This keeps depth readable on both dark and light palettes.
pub(crate) fn push_dot(
    out: &mut Vec<Particle>,
    x: f32,
    y: f32,
    depth: f32,
    radius: f32,
    white: f32,
    alpha: f32,
) {
    if !alpha.is_finite() || alpha < 0.02 {
        return;
    }
    let white = white.clamp(0.0, 1.0);
    let alpha = alpha.clamp(0.0, 1.0);
    let intensity = (1.0 - white) * alpha;
    out.push(Particle::new(
        x,
        y,
        depth.clamp(-1.0, 1.0),
        radius.max(0.0),
        intensity,
    ));
}

/// Rotates a point around one of the three cardinal axes.
pub(crate) fn rotate_axis(point: Vec3, axis: u8, angle: f64) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    let (sin, cos) = (sin as f32, cos as f32);
    match axis {
        0 => Vec3::new(
            point.x,
            point.y * cos - point.z * sin,
            point.y * sin + point.z * cos,
        ),
        1 => Vec3::new(
            point.x * cos + point.z * sin,
            point.y,
            -point.x * sin + point.z * cos,
        ),
        _ => Vec3::new(
            point.x * cos - point.y * sin,
            point.x * sin + point.y * cos,
            point.z,
        ),
    }
}

/// A point on a circle in the XY plane, used by idle.
pub(crate) fn circle_point(radius: f32, angle: f64) -> Vec3 {
    Vec3::new(
        (angle.cos() as f32) * radius,
        (angle.sin() as f32) * radius,
        0.0,
    )
}

/// The angle for one point in a closed loop.
pub(crate) fn loop_angle(index: usize, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        std::f64::consts::TAU / count as f64 * index as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_points_lie_on_the_unit_sphere() {
        for index in 0..64 {
            let point = fib_dir(index, 64);
            assert!((point.length() - 1.0).abs() < 1.0e-5);
        }
    }

    #[test]
    fn rotations_preserve_length() {
        let point = Vec3::new(0.3, -0.7, 0.5);
        let rotated = rotate_axis(rotate_axis(rotate_axis(point, 0, 0.7), 1, -1.2), 2, 2.3);
        assert!((rotated.length() - point.length()).abs() < 1.0e-5);
    }

    #[test]
    fn hash_is_stable_and_bounded() {
        for index in 0..100 {
            let value = hash_d(f64::from(index), 1.7);
            assert!((0.0..1.0).contains(&value));
            assert_eq!(value, hash_d(f64::from(index), 1.7));
        }
    }

    #[test]
    fn smoothstep_is_clamped() {
        assert_eq!(smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 0.5), 0.5);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
        assert_eq!(smoothstep(0.0, 1.0, 2.0), 1.0);
    }
}
