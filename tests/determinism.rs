use std::time::Duration;

use thinking_orbs_rs::{ConfigError, OrbAnimator, OrbConfig, OrbSize, OrbState};

fn animator(state: OrbState) -> OrbAnimator {
    let mut animator = OrbAnimator::new(OrbConfig {
        size: OrbSize::Compact,
        speed: 1.3,
        ..OrbConfig::default()
    });
    animator.set_state(state);
    animator
}

#[test]
fn identical_inputs_produce_identical_frames() {
    for state in OrbState::ALL {
        let animator = animator(state);
        let first = animator.frame_at(2.75);
        let second = animator.frame_at(2.75);
        assert_eq!(first, second, "state {state:?} was not deterministic");
    }
}

#[test]
fn frame_at_does_not_depend_on_internal_clock() {
    let mut animator = animator(OrbState::Working);
    let expected = animator.frame_at(1.5);
    let _ = animator.tick(9.0);
    assert_eq!(animator.frame_at(1.5), expected);
}

#[test]
fn tick_matches_frame_at_for_the_same_elapsed_time() {
    let mut animator = animator(OrbState::Searching);
    let _ = animator.tick(0.75);
    let ticked = animator.tick(0.0).clone();
    assert_eq!(ticked, animator.frame_at(0.75));
}

#[test]
fn tick_reuses_the_internal_frame_buffer() {
    let mut animator = animator(OrbState::Weaving);
    let first = animator.tick(0.016) as *const _;
    let second = animator.tick(0.016) as *const _;
    assert_eq!(first, second);
}

#[test]
fn current_frame_matches_the_last_tick() {
    let mut animator = animator(OrbState::Listening);
    let ticked = animator.tick(0.5).clone();
    assert_eq!(animator.frame(), &ticked);
}

#[test]
fn frame_interval_tracks_fps() {
    let config = OrbConfig {
        fps: 20,
        ..OrbConfig::default()
    };
    assert_eq!(config.frame_interval(), Duration::from_millis(50));

    let clamped = OrbConfig {
        fps: 0,
        ..OrbConfig::default()
    };
    assert_eq!(clamped.frame_interval(), Duration::from_secs(1));
}

#[test]
fn pause_freezes_the_clock() {
    let mut animator = animator(OrbState::Breathing);
    let _ = animator.tick(1.0);
    animator.pause();
    let frozen = animator.frame_at(animator.elapsed());
    let _ = animator.tick(10.0);
    assert_eq!(animator.elapsed(), 1.0);
    assert_eq!(animator.tick(0.0), &frozen);
    animator.resume();
    let _ = animator.tick(0.5);
    assert_eq!(animator.elapsed(), 1.5);
}

#[test]
fn reset_returns_to_time_zero() {
    let mut animator = animator(OrbState::Shaping);
    let _ = animator.tick(2.0);
    animator.reset();
    assert_eq!(animator.elapsed(), 0.0);
    let expected = animator.frame_at(0.0);
    assert_eq!(animator.tick(0.0), &expected);
}

#[test]
fn reduced_motion_is_frozen() {
    let config = OrbConfig {
        reduced_motion: true,
        ..OrbConfig::default()
    };
    let animator = OrbAnimator::new(config);
    assert_eq!(animator.frame_at(0.0), animator.frame_at(120.0));
}

#[test]
fn invalid_time_is_treated_as_zero() {
    let animator = animator(OrbState::Connecting);
    assert_eq!(animator.frame_at(-5.0), animator.frame_at(0.0));
    assert_eq!(animator.frame_at(f64::NAN), animator.frame_at(0.0));
    assert_eq!(animator.frame_at(f64::INFINITY), animator.frame_at(0.0));
}

#[test]
fn negative_delta_does_not_move_the_clock() {
    let mut animator = animator(OrbState::Idle);
    let _ = animator.tick(-1.0);
    assert_eq!(animator.elapsed(), 0.0);
}

#[test]
fn configuration_is_validated_and_sanitized() {
    let invalid = OrbConfig {
        speed: f32::NAN,
        fps: 0,
        ..OrbConfig::default()
    };
    assert_eq!(invalid.validate(), Err(ConfigError::NonFiniteSpeed));

    let sanitized = invalid.sanitized();
    assert_eq!(sanitized.speed, 1.0);
    assert_eq!(sanitized.fps, 1);

    let fast = OrbConfig {
        speed: 2.0,
        fps: 200,
        ..OrbConfig::default()
    };
    assert_eq!(fast.validate(), Err(ConfigError::FpsOutOfRange(200)));
    assert_eq!(fast.sanitized().fps, 30);

    let negative = OrbConfig {
        speed: -0.5,
        ..OrbConfig::default()
    };
    assert_eq!(negative.validate(), Err(ConfigError::NonPositiveSpeed));
}
