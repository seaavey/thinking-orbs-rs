use crate::config::OrbSize;
use crate::particle::Particle;

/// The result of evaluating one animation frame.
///
/// Coordinates are terminal-cell coordinates. The frame's width and height
/// describe the logical canvas; a renderer may crop it to a smaller terminal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    /// Logical canvas width in cells.
    pub width: u16,
    /// Logical canvas height in cells.
    pub height: u16,
    /// Particles sorted from back to front.
    pub particles: Vec<Particle>,
}

impl Frame {
    /// Creates an empty frame with the given logical dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            particles: Vec::new(),
        }
    }

    /// Returns true when the frame has no particles.
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    /// Scales normalized particles into cell coordinates and sorts them for
    /// back-to-front drawing.
    pub(crate) fn prepare(&mut self, size: OrbSize) {
        let (width, height) = size.dimensions();
        self.width = width;
        self.height = height;

        let center_x = (f32::from(width) - 1.0) * 0.5;
        let center_y = (f32::from(height) - 1.0) * 0.5;
        let radius_x = f32::from(width) * 0.5;
        let radius_y = f32::from(height) * 0.5;

        for particle in &mut self.particles {
            particle.x = center_x + particle.x * radius_x;
            particle.y = center_y + particle.y * radius_y;
            particle.radius = (0.55 + particle.radius * 0.9).clamp(0.4, 2.2);
            particle.depth = particle.depth.clamp(-1.0, 1.0);
            particle.intensity = particle.intensity.clamp(0.0, 1.0);
        }

        self.particles
            .sort_by(|left, right| left.depth.total_cmp(&right.depth));
    }
}
