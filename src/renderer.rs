use std::fmt::Write as _;

use crate::config::{OrbStyle, OrbTheme};
use crate::frame::Frame;

/// Maps a frame to terminal characters.
///
/// The renderer is stateless apart from its style, theme, and color switch.
/// Color is opt-in because redirected output and non-TTY environments must not
/// receive ANSI escapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrbRenderer {
    style: OrbStyle,
    theme: OrbTheme,
    color: bool,
}

impl OrbRenderer {
    /// Creates a plain-text renderer. Call [OrbRenderer::with_color] to enable
    /// ANSI colors.
    pub fn new(style: OrbStyle, theme: OrbTheme) -> Self {
        Self {
            style,
            theme: theme.resolved(),
            color: false,
        }
    }

    /// Enables or disables ANSI color output.
    pub const fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Renders the full logical frame.
    pub fn render(&self, frame: &Frame) -> String {
        self.render_viewport(frame, frame.width, frame.height)
    }

    /// Renders a clipped viewport. This is used when the terminal is smaller
    /// than the frame's logical canvas.
    pub fn render_viewport(&self, frame: &Frame, max_width: u16, max_height: u16) -> String {
        let width = frame.width.min(max_width);
        let height = frame.height.min(max_height);
        if width == 0 || height == 0 {
            return String::new();
        }

        let width_usize = usize::from(width);
        let height_usize = usize::from(height);
        let mut cells = vec![Cell::default(); width_usize * height_usize];

        for particle in &frame.particles {
            let x = particle.x.round();
            let y = particle.y.round();
            if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= width_usize || y >= height_usize {
                continue;
            }

            let weight = particle.visual_weight();
            if !weight.is_finite() {
                continue;
            }
            let level = level_for(weight);
            let index = y * width_usize + x;
            let cell = &mut cells[index];
            if weight > cell.weight || (weight == cell.weight && particle.depth > cell.depth) {
                cell.ch = glyph(self.style, level);
                cell.level = level;
                cell.weight = weight;
                cell.depth = particle.depth;
            }
        }

        let mut output = String::with_capacity(width_usize * height_usize + height_usize);
        for row in 0..height_usize {
            let start = row * width_usize;
            let mut line = String::with_capacity(width_usize * 4);
            for cell in &cells[start..start + width_usize] {
                if cell.ch == ' ' {
                    line.push(' ');
                } else if self.color {
                    let code = color_for(self.theme, cell.level);
                    let _ = write!(line, "\x1b[38;5;{code}m{}\x1b[0m", cell.ch);
                } else {
                    line.push(cell.ch);
                }
            }
            output.push_str(line.trim_end());
            if row + 1 < height_usize {
                output.push('\n');
            }
        }
        output
    }
}

#[derive(Clone, Copy, Debug)]
struct Cell {
    ch: char,
    level: u8,
    weight: f32,
    depth: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            level: 0,
            weight: 0.0,
            depth: f32::NEG_INFINITY,
        }
    }
}

fn level_for(weight: f32) -> u8 {
    match weight {
        w if w < 0.22 => 1,
        w if w < 0.48 => 2,
        w if w < 0.74 => 3,
        _ => 4,
    }
}

fn glyph(style: OrbStyle, level: u8) -> char {
    match style {
        OrbStyle::Unicode => match level {
            1 => '·',
            2 => '∘',
            3 => '•',
            _ => '●',
        },
        OrbStyle::Ascii => match level {
            1 => '.',
            2 => 'o',
            3 => 'O',
            _ => '@',
        },
    }
}

fn color_for(theme: OrbTheme, level: u8) -> u8 {
    // A deliberately monochrome grayscale ramp: dim gray for the back of the
    // orb and near-white for the front. No hue is introduced.
    match theme {
        OrbTheme::Light => match level {
            1 => 244,
            2 => 240,
            3 => 236,
            _ => 232,
        },
        _ => match level {
            1 => 240,
            2 => 245,
            3 => 250,
            _ => 255,
        },
    }
}
