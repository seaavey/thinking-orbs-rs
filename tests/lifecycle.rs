use std::collections::HashSet;

use thinking_orbs_rs::{
    AgentEvent, OrbAnimator, OrbConfig, OrbState, handle_agent_event, state_for_event,
    state_for_tool,
};

#[test]
fn lifecycle_events_map_to_expected_states() {
    let cases = [
        (AgentEvent::RunStarted, OrbState::Working),
        (AgentEvent::ProviderConnecting, OrbState::Connecting),
        (AgentEvent::ProviderConnected, OrbState::Working),
        (AgentEvent::ToolFinished, OrbState::Working),
        (AgentEvent::ResponseStarted, OrbState::Composing),
        (AgentEvent::ResponseFinished, OrbState::Idle),
        (AgentEvent::AwaitingInput, OrbState::Listening),
        (AgentEvent::Failed, OrbState::Idle),
        (AgentEvent::Cancelled, OrbState::Idle),
    ];

    for (event, expected) in cases {
        assert_eq!(state_for_event(&event), expected, "event {event:?}");
    }
}

#[test]
fn tool_names_cover_specialized_states() {
    let cases = [
        ("web_search", OrbState::Searching),
        ("grep_files", OrbState::Searching),
        ("validate_json", OrbState::Solving),
        ("run_tests", OrbState::Solving),
        ("pipeline", OrbState::Weaving),
        ("format_markdown", OrbState::Shaping),
        ("write_file", OrbState::Composing),
        ("http_request", OrbState::Connecting),
        ("listen_stdin", OrbState::Listening),
        ("think", OrbState::Breathing),
        ("unknown_tool", OrbState::Working),
    ];

    for (name, expected) in cases {
        assert_eq!(state_for_tool(name), expected, "tool {name}");
    }
}

#[test]
fn every_state_is_reachable_from_an_event_or_tool_hint() {
    let events = [
        AgentEvent::RunStarted,
        AgentEvent::ProviderConnecting,
        AgentEvent::ToolStarted {
            name: "web_search".to_owned(),
        },
        AgentEvent::ToolStarted {
            name: "validate".to_owned(),
        },
        AgentEvent::AwaitingInput,
        AgentEvent::ToolStarted {
            name: "pipeline".to_owned(),
        },
        AgentEvent::ResponseStarted,
        AgentEvent::ToolStarted {
            name: "think".to_owned(),
        },
        AgentEvent::ToolStarted {
            name: "format".to_owned(),
        },
        AgentEvent::ResponseFinished,
    ];

    let states: HashSet<OrbState> = events.iter().map(state_for_event).collect();
    assert_eq!(states.len(), OrbState::ALL.len());
}

#[test]
fn handler_updates_the_animator() {
    let mut animator = OrbAnimator::new(OrbConfig::default());
    assert_eq!(animator.state(), OrbState::Idle);

    handle_agent_event(&mut animator, &AgentEvent::RunStarted);
    assert_eq!(animator.state(), OrbState::Working);

    handle_agent_event(
        &mut animator,
        &AgentEvent::ToolStarted {
            name: "web_search".to_owned(),
        },
    );
    assert_eq!(animator.state(), OrbState::Searching);

    handle_agent_event(&mut animator, &AgentEvent::Cancelled);
    assert_eq!(animator.state(), OrbState::Idle);
}
