use crate::algorithms::composing;
use crate::config::OrbSize;
use crate::particle::Particle;

/// A face-on ring whose radius undulates — the reference "breathing" mode.
pub(crate) fn sample(time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    composing::sample_face_on(time, size, out);
}
