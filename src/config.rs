use std::fmt;
use std::time::Duration;

/// The minimum frame rate accepted by [OrbConfig].
pub const MIN_FPS: u8 = 1;
/// The maximum frame rate accepted by [OrbConfig].
pub const MAX_FPS: u8 = 30;

/// The logical size of the orb canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrbSize {
    /// A small canvas suitable for a prompt line.
    Compact,
    /// The default canvas for a full status area.
    #[default]
    Normal,
}

impl OrbSize {
    /// Returns the logical cell dimensions of this size.
    pub const fn dimensions(self) -> (u16, u16) {
        match self {
            Self::Compact => (28, 14),
            Self::Normal => (48, 24),
        }
    }
}

/// The terminal color theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrbTheme {
    /// Resolve automatically. The renderer currently resolves this to
    /// [OrbTheme::Dark] because terminals do not expose their background
    /// color reliably.
    #[default]
    Auto,
    /// Light glyphs for a dark terminal background.
    Dark,
    /// Dark glyphs for a light terminal background.
    Light,
}

impl OrbTheme {
    /// Resolves [OrbTheme::Auto] to a concrete theme.
    pub const fn resolved(self) -> Self {
        match self {
            Self::Auto => Self::Dark,
            other => other,
        }
    }
}

/// The character set used by the renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrbStyle {
    /// Unicode block and dot glyphs.
    #[default]
    Unicode,
    /// An ASCII-only fallback for limited terminals.
    Ascii,
}

/// Configuration for an orb animation.
///
/// The fields are public so an integration can use struct update syntax, but
/// values are sanitized by [OrbAnimator](crate::OrbAnimator) before use. Call
/// [OrbConfig::validate] when an invalid configuration should be rejected
/// instead of repaired.
#[derive(Clone, Debug, PartialEq)]
pub struct OrbConfig {
    /// Logical canvas size.
    pub size: OrbSize,
    /// Color theme.
    pub theme: OrbTheme,
    /// Character set.
    pub style: OrbStyle,
    /// Animation speed multiplier. Must be finite and greater than zero.
    pub speed: f32,
    /// Frames per second. Values outside [MIN_FPS]..=[MAX_FPS] are clamped.
    pub fps: u8,
    /// When true, the animation is frozen at its resting pose.
    pub reduced_motion: bool,
}

impl Default for OrbConfig {
    fn default() -> Self {
        Self {
            size: OrbSize::Normal,
            theme: OrbTheme::Auto,
            style: OrbStyle::Unicode,
            speed: 1.0,
            fps: 24,
            reduced_motion: false,
        }
    }
}

impl OrbConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.speed.is_finite() {
            return Err(ConfigError::NonFiniteSpeed);
        }
        if self.speed <= 0.0 {
            return Err(ConfigError::NonPositiveSpeed);
        }
        if !(MIN_FPS..=MAX_FPS).contains(&self.fps) {
            return Err(ConfigError::FpsOutOfRange(self.fps));
        }
        Ok(())
    }

    /// Returns a configuration that is safe to use.
    ///
    /// Invalid speeds fall back to 1.0 and the frame rate is clamped to the
    /// supported range. Other fields are left unchanged.
    pub fn sanitized(&self) -> Self {
        let mut config = self.clone();
        if !config.speed.is_finite() || config.speed <= 0.0 {
            config.speed = 1.0;
        }
        config.fps = config.fps.clamp(MIN_FPS, MAX_FPS);
        config
    }

    /// The interval between frames at the configured frame rate.
    pub fn frame_interval(&self) -> Duration {
        let fps = self.fps.clamp(MIN_FPS, MAX_FPS);
        Duration::from_secs_f64(1.0 / f64::from(fps))
    }
}

/// A validation failure for [OrbConfig].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// speed was NaN or infinite.
    NonFiniteSpeed,
    /// speed was zero or negative.
    NonPositiveSpeed,
    /// fps was outside the supported range.
    FpsOutOfRange(u8),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteSpeed => f.write_str("speed must be finite"),
            Self::NonPositiveSpeed => f.write_str("speed must be greater than zero"),
            Self::FpsOutOfRange(fps) => {
                write!(
                    f,
                    "fps {fps} is outside the supported range {MIN_FPS}-{MAX_FPS}"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}
