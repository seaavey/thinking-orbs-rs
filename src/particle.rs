/// One visual particle in an orb frame.
///
/// Coordinates are expressed in terminal cells after a frame has been
/// prepared. Animation algorithms first produce normalized coordinates in
/// roughly -1.0..=1.0; the frame pipeline projects and scales them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Particle {
    /// Horizontal cell coordinate.
    pub x: f32,
    /// Vertical cell coordinate.
    pub y: f32,
    /// Depth in -1.0..=1.0; positive values are closer to the viewer.
    pub depth: f32,
    /// Visual radius hint, in cell units.
    pub radius: f32,
    /// Brightness in 0.0..=1.0.
    pub intensity: f32,
}

impl Particle {
    /// Creates a particle.
    pub const fn new(x: f32, y: f32, depth: f32, radius: f32, intensity: f32) -> Self {
        Self {
            x,
            y,
            depth,
            radius,
            intensity,
        }
    }

    /// A stable visual score used by renderers to pick a glyph.
    ///
    /// The reference engine paints each dot with ink and alpha; a terminal cell
    /// can only carry one glyph, so the score collapses that pair into the same
    /// intensity value. Depth and radius stay available for sorting and future
    /// renderers but do not brighten a dot on their own.
    pub fn visual_weight(&self) -> f32 {
        self.intensity.clamp(0.0, 1.0)
    }
}
