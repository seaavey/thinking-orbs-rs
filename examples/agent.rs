//! A small terminal AI-agent UI driven by `thinking-orbs-rs`.
//!
//! The demo plays a scripted agent run: connect, plan, search, solve, stream a
//! response, format, idle. The orb is the only animated element; the transcript
//! is plain text. Run it in a real terminal:
//!
//! ```sh
//! cargo run --example agent
//! cargo run --example agent -- --ascii
//! cargo run --example agent -- --static
//! ```
//!
//! Keys: `space` pauses the orb, `q`/`Esc`/`Ctrl+C` quits.

use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Attribute, ResetColor, SetAttribute};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};

use thinking_orbs_rs::{
    OrbAnimator, OrbConfig, OrbRenderer, OrbSize, OrbState, OrbStyle, OrbTheme,
};

const LOOP_SECONDS: f64 = 26.0;
const ORB_HEIGHT: u16 = 14;
const ORB_WIDTH: u16 = 28;

fn main() -> io::Result<()> {
    let static_mode = std::env::args().any(|argument| argument == "--static");
    let ascii = std::env::args().any(|argument| argument == "--ascii");

    if static_mode || !io::stdout().is_terminal() {
        print_static();
        return Ok(());
    }

    run(ascii)
}

fn run(ascii: bool) -> io::Result<()> {
    let _screen = ScreenGuard::enter()?;

    let style = if ascii {
        OrbStyle::Ascii
    } else {
        OrbStyle::Unicode
    };
    let mut animator = OrbAnimator::new(OrbConfig {
        size: OrbSize::Compact,
        style,
        ..OrbConfig::default()
    });
    let renderer = OrbRenderer::new(style, OrbTheme::Dark);

    let started = Instant::now();
    let mut last = Instant::now();
    let mut paused = false;
    let frame_interval = Duration::from_millis(33);

    loop {
        let now = Instant::now();
        let delta = now.saturating_duration_since(last);
        last = now;

        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc | KeyCode::Char('q'),
                    ..
                }) => return Ok(()),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers,
                    ..
                }) if modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    ..
                }) => {
                    paused = !paused;
                    if paused {
                        animator.pause();
                    } else {
                        animator.resume();
                    }
                }
                _ => {}
            }
        }

        let elapsed = started.elapsed().as_secs_f64();
        let t = elapsed % LOOP_SECONDS;
        let scene = scene_at(t);
        animator.set_state(scene.state);
        animator.tick(delta.as_secs_f64());

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        // Some multiplexers report 0x0 before the first resize event.
        let (cols, rows) = if cols == 0 || rows == 0 {
            (80, 24)
        } else {
            (cols, rows)
        };
        let layout = Layout::new(cols, rows);
        let frame = renderer.render(animator.frame());
        let orb: Vec<&str> = frame.split('\n').collect();
        let screen = compose(&layout, &scene, &orb, paused, t, ascii);

        let mut stdout = io::stdout().lock();
        for (y, row) in screen.iter().enumerate() {
            queue!(
                stdout,
                MoveTo(0, y as u16),
                Clear(ClearType::CurrentLine),
                SetAttribute(Attribute::Reset)
            )?;
            write!(stdout, "{row}")?;
        }
        stdout.flush()?;

        if let Some(rest) = frame_interval.checked_sub(now.elapsed()) {
            thread::sleep(rest);
        }
    }
}

fn print_static() {
    let scene = scene_at(22.0);
    println!("SEAVX AGENT · thinking-orbs-rs demo");
    println!();
    for line in &scene.log {
        match line.kind {
            Kind::User => println!("you  {}", line.text),
            Kind::Tool => println!("  · {}", line.text),
            Kind::Note => println!("  {}", line.text),
            Kind::Assistant => println!("  {}", line.text),
        }
    }
    println!();
    println!("[{}] done", scene.state.label());
    println!();
    println!("Run this example in a terminal to see the orb animate.");
}

#[derive(Clone, Copy)]
enum Kind {
    User,
    Tool,
    Note,
    Assistant,
}

struct Line {
    kind: Kind,
    text: String,
}

struct Scene {
    state: OrbState,
    log: Vec<Line>,
}

fn scene_at(t: f64) -> Scene {
    let mut log = Vec::new();
    let push = |log: &mut Vec<Line>, at: f64, kind: Kind, text: &str| {
        if t >= at {
            log.push(Line {
                kind,
                text: text.to_owned(),
            });
        }
    };

    push(
        &mut log,
        0.3,
        Kind::User,
        "refactor the parser and run the tests",
    );
    push(
        &mut log,
        1.0,
        Kind::Note,
        "provider connected - deepseek-v4",
    );
    push(&mut log, 2.2, Kind::Tool, "Working  planning the change");
    push(&mut log, 3.0, Kind::Tool, "Working  read Cargo.toml");
    push(&mut log, 3.8, Kind::Note, "3 crates in workspace");
    push(
        &mut log,
        4.6,
        Kind::Tool,
        "Searching  grep -rn \"parse_\" src/",
    );
    push(&mut log, 5.1, Kind::Note, "7 files - 23 matches");
    push(
        &mut log,
        5.8,
        Kind::Tool,
        "Searching  read src/parser/mod.rs",
    );
    push(
        &mut log,
        6.3,
        Kind::Tool,
        "Searching  read src/parser/ast.rs",
    );
    push(
        &mut log,
        7.0,
        Kind::Tool,
        "Searching  read src/parser/lexer.rs",
    );
    push(&mut log, 8.6, Kind::Tool, "Solving  cargo test --workspace");
    push(
        &mut log,
        9.6,
        Kind::Tool,
        "Solving  cargo clippy --all-targets",
    );
    push(&mut log, 10.4, Kind::Note, "12 passed - 0 failed");
    push(&mut log, 11.2, Kind::Note, "no warnings");

    let reply = "I moved token scanning into src/parser/lexer.rs, kept the public AST unchanged, and the workspace suite still passes.";
    let revealed = ((t - 12.1) * 55.0).max(0.0) as usize;
    if t >= 12.1 {
        let text: String = reply.chars().take(revealed).collect();
        log.push(Line {
            kind: Kind::Assistant,
            text,
        });
    }

    push(
        &mut log,
        18.2,
        Kind::Tool,
        "Shaping  formatting the summary",
    );
    push(&mut log, 19.2, Kind::Note, "1 file changed - +24 -18");
    push(&mut log, 21.0, Kind::Note, "done - 3 tools - 0 errors");

    Scene {
        state: state_at(t),
        log,
    }
}

fn state_at(t: f64) -> OrbState {
    if t < 2.0 {
        OrbState::Connecting
    } else if t < 4.5 {
        OrbState::Working
    } else if t < 8.5 {
        OrbState::Searching
    } else if t < 12.0 {
        OrbState::Solving
    } else if t < 18.0 {
        OrbState::Composing
    } else if t < 21.0 {
        OrbState::Shaping
    } else {
        OrbState::Idle
    }
}

struct Layout {
    width: u16,
    orb_x: u16,
    orb_y: u16,
    log_x: u16,
    log_y: u16,
    log_w: u16,
    log_h: u16,
    status_y: u16,
}

impl Layout {
    fn new(cols: u16, rows: u16) -> Self {
        let status_y = rows.saturating_sub(1);
        if cols >= 66 && rows >= 18 {
            Self {
                width: cols,
                orb_x: 2,
                orb_y: 2,
                log_x: 34,
                log_y: 2,
                log_w: cols - 36,
                log_h: rows.saturating_sub(4),
                status_y,
            }
        } else {
            Self {
                width: cols,
                orb_x: 1,
                orb_y: 1,
                log_x: 1,
                log_y: 16,
                log_w: cols.saturating_sub(2),
                log_h: rows.saturating_sub(17),
                status_y,
            }
        }
    }
}

fn compose(
    layout: &Layout,
    scene: &Scene,
    orb: &[&str],
    paused: bool,
    t: f64,
    ascii: bool,
) -> Vec<String> {
    let height = usize::from(layout.status_y) + 1;
    let width = usize::from(layout.width);
    let separator = if ascii { "|" } else { "·" };
    let mut screen = vec![String::new(); height];

    screen[0] = styled(
        Attribute::Bold,
        &format!(" SEAVX AGENT {separator} thinking-orbs-rs"),
    );

    for (index, line) in orb.iter().take(usize::from(ORB_HEIGHT)).enumerate() {
        let y = usize::from(layout.orb_y) + index;
        if y >= height {
            break;
        }
        let mut row = " ".repeat(usize::from(layout.orb_x));
        row.push_str(&truncate(line, usize::from(ORB_WIDTH)));
        screen[y] = row;
    }

    let rows = render_log(&scene.log, layout.log_w, ascii);
    let visible = usize::from(layout.log_h);
    let first = rows.len().saturating_sub(visible);
    for (index, row) in rows.iter().skip(first).enumerate() {
        let y = usize::from(layout.log_y) + index;
        if y >= height {
            break;
        }
        let current = &mut screen[y];
        let used = current.chars().count();
        let pad = usize::from(layout.log_x).saturating_sub(used);
        current.push_str(&" ".repeat(pad));
        current.push_str(row);
    }

    let status = format!(
        " {}  {separator}  {:02}:{:02}  {separator}  {}  {separator}  space pause {separator} q quit",
        scene.state.label(),
        (t as u64) / 60,
        (t as u64) % 60,
        if paused { "paused" } else { "running" }
    );
    let status = format!("{status:<width$}");
    screen[usize::from(layout.status_y)] = styled(Attribute::Reverse, &truncate(&status, width));

    screen
}

fn render_log(log: &[Line], width: u16, ascii: bool) -> Vec<String> {
    let width = usize::from(width);
    let bullet = if ascii { "  - " } else { "  · " };
    let mut rows = Vec::new();

    for line in log {
        let (prefix, prefix_width, body_width) = match line.kind {
            Kind::User => (styled(Attribute::Bold, "you  "), 5, width.saturating_sub(5)),
            Kind::Tool => (styled(Attribute::Dim, bullet), 4, width.saturating_sub(4)),
            Kind::Note => (String::new(), 2, width.saturating_sub(2)),
            Kind::Assistant => ("  ".to_owned(), 2, width.saturating_sub(2)),
        };

        for (index, chunk) in wrap(&line.text, body_width.max(8)).into_iter().enumerate() {
            let row = if index == 0 {
                format!("{prefix}{chunk}")
            } else {
                format!("{}{chunk}", " ".repeat(prefix_width))
            };
            rows.push(match line.kind {
                Kind::Note => styled(Attribute::Dim, &row),
                _ => row,
            });
        }
    }

    rows
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            rows.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }
    rows
}

fn truncate(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn styled(attribute: Attribute, text: &str) -> String {
    format!(
        "{}{text}{}",
        SetAttribute(attribute),
        SetAttribute(Attribute::Reset)
    )
}

struct ScreenGuard;

impl ScreenGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for ScreenGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            Show,
            LeaveAlternateScreen,
            ResetColor,
            SetAttribute(Attribute::Reset)
        );
        let _ = terminal::disable_raw_mode();
    }
}
