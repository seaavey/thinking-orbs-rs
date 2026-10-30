# thinking-orbs-rs implementation notes

This document records the design decisions behind the crate. The public API is
documented with rustdoc; this file focuses on the animation pipeline, terminal
integration, and the limits of the current implementation.

## 1. Scope

The crate provides:

- nine animations plus an idle state;
- deterministic frame generation;
- Unicode and ASCII rendering;
- a monochrome white/gray ANSI palette, only when explicitly enabled;
- an agent lifecycle adapter;
- an inline terminal adapter with cursor cleanup;
- frame-rate limiting and reduced motion;
- integration tests for states, determinism, lifecycle, and rendering.

It does not provide a new agent runtime, provider protocol changes, a web UI, a
Canvas renderer, WebAssembly, or GPU rendering.

## 2. Module map

| Module | Responsibility |
| --- | --- |
| `state` | `OrbState` enum and labels |
| `config` | Size, theme, style, speed, FPS, reduced motion, validation |
| `geometry` | 3D points, rotation, perspective, interpolation, hashes, shapes |
| `particle` | Particle data and visual weight |
| `frame` | One evaluated frame and normalization into cell coordinates |
| `animator` | State, clock, reusable buffer, `frame_at`, `tick` |
| `algorithms` | Nine animations plus idle |
| `renderer` | Frame to Unicode/ASCII text, optional ANSI color, viewport crop |
| `terminal` | Inline redraw, TTY detection, cursor guard |
| `lifecycle` | `AgentEvent` to `OrbState` mapping |

The animation core is independent of `crossterm`. Only `terminal` uses
`crossterm`.

## 3. Coordinate pipeline

Algorithms work in a local space roughly inside `[-1, 1]^3`:

1. Generate normalized positions.
2. Apply rotations and a mild perspective projection.
3. Convert the projected point to a `Particle` with depth and intensity.
4. `Frame::prepare` scales x and y into the logical cell canvas, using a
   wider x radius so the character grid's 2:1 cell aspect looks round.
5. Particles are sorted from back to front by depth.
6. `OrbRenderer` picks the brightest particle per cell and maps its visual
   weight to a glyph.

The logical canvas is `28x14` for `Compact` and `48x24` for `Normal`.
The renderer can crop to a smaller terminal viewport without changing the
animator.

## 4. Animation notes

All formulas use a local time `t` already multiplied by the configured speed.
No algorithm reads a clock or random source.

### Idle

Two slowly drifting rings with a very small breathing radius change and a
low-intensity core. The motion is deliberately quieter than `Breathing`.

### Working

Five circles are tilted on different axes and spun at different rates. A small
deterministic nucleus keeps the intersections readable as one object.

### Searching

A Fibonacci sphere is sampled at 320 points. A scan longitude advances over
time; each point's intensity is a Gaussian falloff from that meridian. The
sphere itself rotates slowly so the scan does not look static.

### Solving

Target points come from a Fibonacci sphere. A deterministic per-index noise
vector defines a scrambled position. A raised-cosine phase moves the cloud from
resolved to scrambled and back, starting and ending at the resolved sphere so
the loop has no jump.

### Listening

Three concentric rings carry travelling sine waves. The radius varies along the
ring angle, so the shape ripples rather than expanding symmetrically.

### Connecting

Twenty-six nodes come from a Fibonacci sphere. The shortest node pairs become
edges, capped at 34 so the terminal stays legible. Edges are revealed in order
during each six-second cycle, then faded out before the cycle restarts.

### Weaving

Three strands use a shared spherical parameterization with different phase and
latitude frequencies. Depth sorting makes the strands cross over and under one
another.

### Composing

Four latitude ribbons wrap the sphere. Each has a different angular frequency
and a slowly drifting amplitude and phase.

### Breathing

Two concentric rings expand and contract with a slow sine wave. Every point at
a given radius shares the same phase, unlike `Listening`.

### Shaping

Circle, triangle, and square perimeters are sampled at equal arc-length
intervals. The animation morphs circle -> triangle -> square -> circle with a
cubic ease-in/ease-out.

## 5. Lifecycle integration

`AgentEvent` is intentionally small and provider-neutral. The recommended
integration is:

1. Translate the host's existing agent event into `AgentEvent`.
2. Call `handle_agent_event` or `state_for_event`.
3. If the host has a more precise stage, call `OrbAnimator::set_state`
   directly.

Tool-name mapping is a hint. For example, a tool named `web_search` maps to
`Searching`, but a host that knows a tool is actually a local grep should send
the explicit state instead. This avoids making a naming convention the only
source of truth.

## 6. Terminal safety

`TerminalOrb`:

- checks `stdout().is_terminal()`;
- hides the cursor only in a TTY;
- redraws only its own line region;
- clears leftover lines when the new frame is shorter;
- restores cursor visibility in `clear`, `Drop`, and panic unwind;
- disables color when `NO_COLOR` is set;
- coalesces draws to the configured frame interval, so a 60 Hz host tick does
  not produce 60 FPS output;
- does not enter raw mode, so it does not compete with the host's input loop.

The host owns SIGINT handling. If the process is terminated by a signal, normal
destructors do not run; call `clear` from the host's signal/cancellation path
when that path is available. A TUI that already owns stdout should use the
animator and renderer directly instead of creating a second `TerminalOrb`.

## 7. Testing strategy

- `tests/determinism.rs`: same inputs, pause/resume, reset, reduced motion,
  invalid time, buffer reuse, configuration validation.
- `tests/animation_states.rs`: all states produce finite particles, distinct
  visual signatures, stable counts, and valid canvas sizes.
- `tests/lifecycle.rs`: event mapping, tool hints, all states reachable, handler
  updates.
- `tests/rendering.rs`: ASCII fallback, Unicode glyphs, opt-in color, viewport
  clipping, empty viewports.

Manual checks should include an interactive terminal, a narrow terminal,
redirected output, provider failure, cancellation, and a long-running tool.

## 8. Attribution

Copyright © 2026 MUHAMMAD ADRIANSYAH, released under the MIT license.

The visual direction is inspired by
[Jakubantalik/thinking-orbs](https://github.com/Jakubantalik/thinking-orbs),
which is MIT-licensed. This Rust crate ports that engine's geometry, presets,
and timing to terminal character grids; the rendering layer is new because a
terminal cannot draw sub-pixel dots or stroked lines. The reference project's
copyright and license notice are preserved in this repository.
