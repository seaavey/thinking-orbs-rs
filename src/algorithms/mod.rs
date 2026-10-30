mod breathing;
mod composing;
mod connecting;
mod idle;
mod listening;
mod searching;
mod shaping;
mod solving;
mod weaving;
mod working;

use crate::config::OrbSize;
use crate::particle::Particle;
use crate::state::OrbState;

pub(crate) fn sample(state: OrbState, time: f64, size: OrbSize, out: &mut Vec<Particle>) {
    out.clear();
    match state {
        OrbState::Idle => idle::sample(time, size, out),
        OrbState::Working => working::sample(time, size, out),
        OrbState::Searching => searching::sample(time, size, out),
        OrbState::Solving => solving::sample(time, size, out),
        OrbState::Listening => listening::sample(time, size, out),
        OrbState::Connecting => connecting::sample(time, size, out),
        OrbState::Weaving => weaving::sample(time, size, out),
        OrbState::Composing => composing::sample(time, size, out),
        OrbState::Breathing => breathing::sample(time, size, out),
        OrbState::Shaping => shaping::sample(time, size, out),
    }
}
