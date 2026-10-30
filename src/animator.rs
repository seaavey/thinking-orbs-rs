use crate::algorithms;
use crate::config::OrbConfig;
use crate::frame::Frame;
use crate::preset;
use crate::state::OrbState;

/// Deterministic animation driver.
///
/// The animator owns a reusable frame buffer. [OrbAnimator::frame_at] is pure
/// with respect to its inputs; [OrbAnimator::tick] advances the internal clock
/// and writes into the existing buffer.
#[derive(Clone, Debug)]
pub struct OrbAnimator {
    state: OrbState,
    config: OrbConfig,
    elapsed: f64,
    paused: bool,
    frame: Frame,
}

impl OrbAnimator {
    /// Creates an animator in [OrbState::Idle].
    ///
    /// The configuration is sanitized before it is stored.
    pub fn new(config: OrbConfig) -> Self {
        let config = config.sanitized();
        let (width, height) = config.size.dimensions();
        Self {
            state: OrbState::Idle,
            config,
            elapsed: 0.0,
            paused: false,
            frame: Frame::new(width, height),
        }
    }

    /// Changes the visible state. The animation clock is not reset.
    pub fn set_state(&mut self, state: OrbState) {
        self.state = state;
    }

    /// Returns the current state.
    pub fn state(&self) -> OrbState {
        self.state
    }

    /// Returns the sanitized configuration.
    pub fn config(&self) -> &OrbConfig {
        &self.config
    }

    /// Replaces the configuration. The clock and state are preserved.
    pub fn set_config(&mut self, config: OrbConfig) {
        self.config = config.sanitized();
        let (width, height) = self.config.size.dimensions();
        self.frame.width = width;
        self.frame.height = height;
    }

    /// Stops the internal clock from advancing in [OrbAnimator::tick].
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the internal clock.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Resets the internal clock to zero. The paused flag is preserved.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Returns the current internal elapsed time in seconds.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Computes a frame without mutating the animator or reading a clock.
    ///
    /// Negative or non-finite times are treated as zero. Reduced motion renders
    /// the reference static pose at t = 0.6 while leaving the frame deterministic.
    pub fn frame_at(&self, elapsed_seconds: f64) -> Frame {
        let (width, height) = self.config.size.dimensions();
        let mut frame = Frame::new(width, height);
        Self::fill_frame(self.state, &self.config, elapsed_seconds, &mut frame);
        frame
    }

    /// Returns the most recently computed frame without advancing the clock.
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Advances the internal clock and returns the reusable frame buffer.
    ///
    /// Negative or non-finite deltas are ignored. When paused, the clock does
    /// not advance, but the frame is still recomputed.
    pub fn tick(&mut self, delta_seconds: f64) -> &Frame {
        if !self.paused && delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.elapsed += delta_seconds;
        }
        Self::fill_frame(self.state, &self.config, self.elapsed, &mut self.frame);
        &self.frame
    }

    fn fill_frame(state: OrbState, config: &OrbConfig, elapsed: f64, frame: &mut Frame) {
        let time = if config.reduced_motion {
            0.6
        } else if !elapsed.is_finite() {
            0.0
        } else {
            let preset_speed = preset::for_state(state, config.size).speed;
            elapsed.max(0.0) * f64::from(config.speed) * preset_speed
        };
        algorithms::sample(state, time, config.size, &mut frame.particles);
        frame.prepare(config.size);
    }
}
