use std::io::{self, IsTerminal};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use thinking_orbs_rs::{OrbConfig, OrbRenderer, OrbSize, OrbState, TerminalOrb};

fn main() -> io::Result<()> {
    let static_preview = std::env::args().any(|argument| argument == "--static");
    if static_preview {
        return print_static_preview();
    }

    if !io::stdout().is_terminal() {
        println!("thinking-orbs-rs gallery requires a TTY");
        println!("States: {}", state_names());
        println!("Use --static for a plain-text preview.");
        return Ok(());
    }

    let _raw = RawModeGuard::enable()?;
    let mut orb = TerminalOrb::new(OrbConfig {
        size: OrbSize::Normal,
        ..OrbConfig::default()
    })?;

    let states = [
        OrbState::Working,
        OrbState::Searching,
        OrbState::Solving,
        OrbState::Listening,
        OrbState::Connecting,
        OrbState::Weaving,
        OrbState::Composing,
        OrbState::Breathing,
        OrbState::Shaping,
        OrbState::Idle,
    ];
    let mut index = 0;
    orb.set_state(states[index]);

    let mut last_tick = Instant::now();
    let mut last_switch = Instant::now();

    loop {
        while event::poll(Duration::from_millis(1))? {
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
                    code: KeyCode::Right,
                    ..
                }) => {
                    index = (index + 1) % states.len();
                    orb.set_state(states[index]);
                    last_switch = Instant::now();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => {
                    index = (index + states.len() - 1) % states.len();
                    orb.set_state(states[index]);
                    last_switch = Instant::now();
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let delta = now.saturating_duration_since(last_tick);
        last_tick = now;
        orb.tick(delta)?;

        if last_switch.elapsed() >= Duration::from_secs(3) {
            index = (index + 1) % states.len();
            orb.set_state(states[index]);
            last_switch = now;
        }

        thread::sleep(Duration::from_millis(16));
    }
}

fn print_static_preview() -> io::Result<()> {
    let renderer = OrbRenderer::new(
        thinking_orbs_rs::OrbStyle::Ascii,
        thinking_orbs_rs::OrbTheme::Dark,
    );
    let states = [
        OrbState::Idle,
        OrbState::Working,
        OrbState::Searching,
        OrbState::Solving,
        OrbState::Listening,
        OrbState::Connecting,
        OrbState::Weaving,
        OrbState::Composing,
        OrbState::Breathing,
        OrbState::Shaping,
    ];

    for state in states {
        let mut animator = thinking_orbs_rs::OrbAnimator::new(OrbConfig {
            size: OrbSize::Compact,
            ..OrbConfig::default()
        });
        animator.set_state(state);
        let frame = animator.frame_at(1.25);
        println!("{}", state.label());
        println!("{}", renderer.render(&frame));
        println!();
    }
    Ok(())
}

fn state_names() -> String {
    OrbState::ALL
        .iter()
        .map(|state| state.label())
        .collect::<Vec<_>>()
        .join(", ")
}

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
