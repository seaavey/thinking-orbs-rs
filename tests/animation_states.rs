use std::collections::HashSet;

use thinking_orbs_rs::{OrbAnimator, OrbConfig, OrbSize, OrbState};

fn frame_for(state: OrbState) -> thinking_orbs_rs::Frame {
    let mut animator = OrbAnimator::new(OrbConfig {
        size: OrbSize::Normal,
        ..OrbConfig::default()
    });
    animator.set_state(state);
    animator.frame_at(1.234)
}

#[test]
fn every_state_produces_finite_particles() {
    for state in OrbState::ALL {
        let frame = frame_for(state);
        assert!(!frame.is_empty(), "state {state:?} produced no particles");
        assert_eq!(frame.width, 48);
        assert_eq!(frame.height, 24);

        for particle in &frame.particles {
            assert!(particle.x.is_finite(), "non-finite x in {state:?}");
            assert!(particle.y.is_finite(), "non-finite y in {state:?}");
            assert!(particle.depth.is_finite(), "non-finite depth in {state:?}");
            assert!(
                particle.radius.is_finite(),
                "non-finite radius in {state:?}"
            );
            assert!(
                particle.intensity.is_finite(),
                "non-finite intensity in {state:?}"
            );
            assert!(
                (-1.0..=1.0).contains(&particle.depth),
                "depth out of range in {state:?}"
            );
            assert!(
                (0.0..=1.0).contains(&particle.intensity),
                "intensity out of range in {state:?}"
            );
            assert!(
                (-16.0..=64.0).contains(&particle.x),
                "x outside the logical canvas in {state:?}: {}",
                particle.x
            );
            assert!(
                (-8.0..=32.0).contains(&particle.y),
                "y outside the logical canvas in {state:?}: {}",
                particle.y
            );
        }
    }
}

#[test]
fn nine_animations_have_distinct_visual_signatures() {
    let mut signatures = HashSet::new();
    for state in OrbState::ALL {
        let frame = frame_for(state);
        let signature: Vec<(i32, i32, i32, i32)> = frame
            .particles
            .iter()
            .map(|particle| {
                (
                    particle.x.round() as i32,
                    particle.y.round() as i32,
                    (particle.depth * 100.0) as i32,
                    (particle.intensity * 100.0) as i32,
                )
            })
            .collect();
        assert!(
            signatures.insert(signature),
            "state {state:?} duplicated another state's visual signature"
        );
    }
    assert_eq!(signatures.len(), OrbState::ALL.len());
}

#[test]
fn particle_counts_are_stable() {
    for state in OrbState::ALL {
        let first = frame_for(state).particles.len();
        let second = frame_for(state).particles.len();
        assert_eq!(first, second, "particle count changed for {state:?}");
    }
}

#[test]
fn labels_are_unique_and_non_empty() {
    let labels: HashSet<&str> = OrbState::ALL.iter().map(|state| state.label()).collect();
    assert_eq!(labels.len(), OrbState::ALL.len());
    assert!(labels.iter().all(|label| !label.is_empty()));
}

#[test]
fn compact_size_is_smaller_than_normal() {
    let mut compact = OrbAnimator::new(OrbConfig {
        size: OrbSize::Compact,
        ..OrbConfig::default()
    });
    compact.set_state(OrbState::Working);
    let compact = compact.frame_at(1.0);

    let mut normal = OrbAnimator::new(OrbConfig::default());
    normal.set_state(OrbState::Working);
    let normal = normal.frame_at(1.0);

    assert!(compact.width < normal.width);
    assert!(compact.height < normal.height);
}
