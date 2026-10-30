//! Deterministic, terminal-native orb animations for the Seavx Agent CLI.
//!
//! The crate is deliberately split into three layers:
//!
//! * a pure animation core ([OrbAnimator]) that turns state, configuration,
//!   and time into a [Frame];
//! * a character renderer ([OrbRenderer]) that maps a frame to terminal
//!   glyphs;
//! * an optional terminal adapter ([TerminalOrb]) that owns cursor cleanup
//!   and the drawing loop.
//!
//! The core never reads the clock, never writes to stdout, and produces the
//! same frame for the same inputs. This makes it safe to drive from a provider
//! stream, a tool executor, or a renderer running on its own thread.
//!
//! # Example
//!
//!     use std::time::Duration;
//!     use thinking_orbs_rs::{AgentEvent, OrbConfig, TerminalOrb};
//!
//!     fn main() -> std::io::Result<()> {
//!         let mut orb = TerminalOrb::new(OrbConfig::default())?;
//!         orb.handle_event(&AgentEvent::RunStarted);
//!         orb.tick(Duration::from_millis(16))?;
//!         Ok(())
//!     }

#![forbid(unsafe_code)]

mod algorithms;
mod geometry;
mod preset;

pub mod animator;
pub mod config;
pub mod frame;
pub mod lifecycle;
pub mod particle;
pub mod renderer;
pub mod state;
pub mod terminal;

pub use animator::OrbAnimator;
pub use config::{ConfigError, OrbConfig, OrbSize, OrbStyle, OrbTheme};
pub use frame::Frame;
pub use lifecycle::{AgentEvent, handle_agent_event, state_for_event, state_for_tool};
pub use particle::Particle;
pub use renderer::OrbRenderer;
pub use state::OrbState;
pub use terminal::TerminalOrb;
