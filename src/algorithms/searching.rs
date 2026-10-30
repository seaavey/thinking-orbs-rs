use crate::config::OrbSize;
use crate::geometry::{Projector, angle_delta_signed, push_dot};
use crate::particle::Particle;
use crate::preset::{for_state, scaled_pair};
use crate::state::OrbState;

/// A dotted globe with a scanning meridian — the reference "searching" mode.
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    let preset = for_state(OrbState::Searching, size);
    let (lat_rings, lon_density) = scaled_pair(17.0, 44.0, preset.count);
    let radius = 0.82;
    let spin = 0.5;
    let tilt = 0.4 + 0.06 * (time * 0.35).sin();
    let projector = Projector::new(time * spin, tilt);
    let scan = time * (spin + (1.7 - spin) * f64::from(preset.scan_mul));
    let base_radius = preset.radius(0.6);
    let depth_radius = preset.radius(1.7);
    let boost_radius = preset.radius(1.0);

    for lat_index in 0..=lat_rings {
        let lat = -std::f64::consts::FRAC_PI_2
            + (lat_index as f64 / lat_rings as f64) * std::f64::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = ((cos_lat.abs() * lon_density as f64).round() as usize).max(1);

        for lon_index in 0..lon_count {
            let lon = (lon_index as f64 / lon_count as f64) * std::f64::consts::TAU;
            let point = (
                (cos_lat * lon.cos()) as f32 * radius,
                sin_lat as f32 * radius,
                (cos_lat * lon.sin()) as f32 * radius,
            );
            let (x, y, z) = projector.project(point.0, point.1, point.2);
            let depth = (z / radius + 1.0) * 0.5;
            let delta = angle_delta_signed(lon + time * spin, scan);
            let boost = (-(delta * delta) / 0.18).exp() * f64::from((z / radius).max(0.0));
            let boost = boost as f32;

            let dot_radius = base_radius + depth_radius * depth + boost_radius * boost;
            let white = 0.62 - 0.54 * depth;
            let alpha = preset.dim_base + (1.0 - preset.dim_base) * boost.min(1.0);
            push_dot(out, x, y, z / radius, dot_radius, white, alpha);
        }
    }
}
