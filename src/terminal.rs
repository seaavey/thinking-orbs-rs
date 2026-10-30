use std::env;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveToColumn, MoveToNextLine, MoveUp, Show};
use crossterm::style::ResetColor;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};

use crate::animator::OrbAnimator;
use crate::config::OrbConfig;
use crate::lifecycle::{AgentEvent, handle_agent_event};
use crate::renderer::OrbRenderer;
use crate::state::OrbState;

/// An inline terminal driver for an orb.
///
/// Construction is a no-op outside a TTY: [TerminalOrb::is_active] returns
/// false and [TerminalOrb::tick] never writes ANSI output. Inside a TTY the
/// driver hides the cursor, redraws only its own region, and restores the
/// cursor when it is dropped or cleared.
pub struct TerminalOrb {
    animator: OrbAnimator,
    renderer: OrbRenderer,
    active: bool,
    drawn_lines: u16,
    frame_interval: Duration,
    draw_accumulator: Duration,
    _guard: Option<TerminalGuard>,
}

impl TerminalOrb {
    /// Creates a terminal driver. This does not clear the screen or print a
    /// banner, so existing conversation output is preserved.
    pub fn new(config: OrbConfig) -> io::Result<Self> {
        let interactive = io::stdout().is_terminal();
        let guard = if interactive {
            Some(TerminalGuard::enter()?)
        } else {
            None
        };
        let color = interactive && env::var_os("NO_COLOR").is_none();
        let renderer = OrbRenderer::new(config.style, config.theme).with_color(color);
        let animator = OrbAnimator::new(config);
        let frame_interval = animator.config().frame_interval();

        Ok(Self {
            animator,
            renderer,
            active: interactive,
            drawn_lines: 0,
            frame_interval,
            draw_accumulator: frame_interval,
            _guard: guard,
        })
    }

    /// Returns true when output is connected to a TTY.
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Replaces the animation configuration and updates the draw throttle.
    pub fn set_config(&mut self, config: OrbConfig) {
        self.animator.set_config(config);
        self.frame_interval = self.animator.config().frame_interval();
        self.draw_accumulator = self.frame_interval;
    }

    /// Changes the visible state.
    pub fn set_state(&mut self, state: OrbState) {
        self.animator.set_state(state);
    }

    /// Applies an agent lifecycle event.
    pub fn handle_event(&mut self, event: &AgentEvent) {
        handle_agent_event(&mut self.animator, event);
    }

    /// Advances the animation and redraws its inline region.
    ///
    /// In a non-TTY this still advances the animator but writes nothing.
    pub fn tick(&mut self, delta: Duration) -> io::Result<()> {
        self.animator.tick(delta.as_secs_f64());
        if !self.active {
            return Ok(());
        }

        self.draw_accumulator = self.draw_accumulator.saturating_add(delta);
        if self.draw_accumulator < self.frame_interval {
            return Ok(());
        }
        self.draw_accumulator = self.draw_accumulator.saturating_sub(self.frame_interval);

        let (columns, rows) = terminal::size().unwrap_or((80, 24));
        let max_width = columns.max(1);
        let max_height = rows.saturating_sub(1).max(1);
        let rendered = {
            let frame = self.animator.frame();
            self.renderer.render_viewport(frame, max_width, max_height)
        };
        self.draw_rendered(&rendered)
    }

    /// Clears the orb region and leaves the cursor at its top-left corner.
    pub fn clear(&mut self) -> io::Result<()> {
        if !self.active || self.drawn_lines == 0 {
            return Ok(());
        }

        let lines = self.drawn_lines;
        let mut stdout = io::stdout().lock();
        queue!(stdout, MoveUp(lines), MoveToColumn(0))?;
        for _ in 0..lines {
            queue!(stdout, Clear(ClearType::CurrentLine), MoveToNextLine(1))?;
        }
        queue!(stdout, MoveUp(lines), MoveToColumn(0))?;
        stdout.flush()?;
        self.drawn_lines = 0;
        Ok(())
    }

    fn draw_rendered(&mut self, rendered: &str) -> io::Result<()> {
        let lines: Vec<&str> = if rendered.is_empty() {
            Vec::new()
        } else {
            rendered.split('\n').collect()
        };
        let line_count = lines.len() as u16;
        let previous = self.drawn_lines;
        let mut stdout = io::stdout().lock();

        if previous > 0 {
            queue!(stdout, MoveUp(previous), MoveToColumn(0))?;
        } else {
            queue!(stdout, MoveToColumn(0))?;
        }
        for line in &lines {
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            write!(stdout, "{line}")?;
            queue!(stdout, MoveToNextLine(1))?;
        }
        if previous > line_count {
            for _ in line_count..previous {
                queue!(stdout, Clear(ClearType::CurrentLine), MoveToNextLine(1))?;
            }
        }
        stdout.flush()?;

        self.drawn_lines = previous.max(line_count);
        Ok(())
    }
}

impl Drop for TerminalOrb {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout().lock();
        execute!(stdout, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout().lock();
        let _ = execute!(stdout, Show, ResetColor);
    }
}
