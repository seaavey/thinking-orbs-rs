use thinking_orbs_rs::{OrbAnimator, OrbConfig, OrbRenderer, OrbState, OrbStyle, OrbTheme};

fn frame_for(state: OrbState) -> thinking_orbs_rs::Frame {
    let mut animator = OrbAnimator::new(OrbConfig::default());
    animator.set_state(state);
    animator.frame_at(0.9)
}

#[test]
fn ascii_rendering_is_plain_and_non_empty() {
    let renderer = OrbRenderer::new(OrbStyle::Ascii, OrbTheme::Dark);
    for state in OrbState::ALL {
        let frame = frame_for(state);
        let rendered = renderer.render(&frame);
        assert!(!rendered.is_empty(), "state {state:?} rendered nothing");
        assert!(rendered.is_ascii(), "state {state:?} emitted non-ASCII");
        assert!(!rendered.contains('\u{1b}'), "state {state:?} emitted ANSI");
        assert!(
            !rendered.contains('\0'),
            "state {state:?} emitted NUL bytes"
        );
        assert_eq!(rendered.split('\n').count(), usize::from(frame.height));
        assert!(
            rendered
                .chars()
                .any(|ch| matches!(ch, '.' | 'o' | 'O' | '@'))
        );
    }
}

#[test]
fn unicode_rendering_uses_unicode_glyphs() {
    let renderer = OrbRenderer::new(OrbStyle::Unicode, OrbTheme::Dark);
    let rendered = renderer.render(&frame_for(OrbState::Searching));
    assert!(
        rendered
            .chars()
            .any(|ch| matches!(ch, '·' | '∘' | '•' | '●'))
    );
    assert!(!rendered.is_ascii());
}

#[test]
fn color_is_opt_in() {
    let frame = frame_for(OrbState::Working);
    let plain = OrbRenderer::new(OrbStyle::Unicode, OrbTheme::Dark).render(&frame);
    let colored = OrbRenderer::new(OrbStyle::Unicode, OrbTheme::Dark)
        .with_color(true)
        .render(&frame);

    assert!(!plain.contains('\u{1b}'));
    assert!(colored.contains("\u{1b}[38;5;"));
    assert!(colored.contains("\u{1b}[0m"));
}

#[test]
fn viewport_clipping_respects_terminal_bounds() {
    let renderer = OrbRenderer::new(OrbStyle::Ascii, OrbTheme::Dark);
    let frame = frame_for(OrbState::Composing);
    let rendered = renderer.render_viewport(&frame, 12, 5);
    assert_eq!(rendered.split('\n').count(), 5);
    assert!(rendered.lines().all(|line| line.chars().count() <= 12));
}

#[test]
fn empty_viewports_are_safe() {
    let renderer = OrbRenderer::new(OrbStyle::Ascii, OrbTheme::Dark);
    let frame = frame_for(OrbState::Idle);
    assert!(renderer.render_viewport(&frame, 0, 0).is_empty());
    assert!(renderer.render_viewport(&frame, 10, 0).is_empty());
    assert!(renderer.render_viewport(&frame, 0, 10).is_empty());
}

#[test]
fn theme_resolution_is_stable() {
    assert_eq!(OrbTheme::Auto.resolved(), OrbTheme::Dark);
    assert_eq!(OrbTheme::Dark.resolved(), OrbTheme::Dark);
    assert_eq!(OrbTheme::Light.resolved(), OrbTheme::Light);
}
