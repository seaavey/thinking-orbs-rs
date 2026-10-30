use crate::animator::OrbAnimator;
use crate::state::OrbState;

/// Events emitted by an agent runtime that can drive an orb.
///
/// This is an integration-facing type. A real Seavx adapter should translate
/// its existing event enum into these values rather than coupling animations
/// to provider code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentEvent {
    /// A run has started.
    RunStarted,
    /// A provider connection is being established or retried.
    ProviderConnecting,
    /// A provider connection is ready.
    ProviderConnected,
    /// A tool has started. The name is a hint, not the only source of truth.
    ToolStarted {
        /// Tool name as reported by the runtime.
        name: String,
    },
    /// A tool has finished.
    ToolFinished,
    /// The model has started producing a response.
    ResponseStarted,
    /// The response has finished.
    ResponseFinished,
    /// The agent is waiting for user input.
    AwaitingInput,
    /// The run failed.
    Failed,
    /// The run was cancelled.
    Cancelled,
}

/// Maps an agent event to the most specific available orb state.
///
/// The mapping is intentionally conservative. Tool names are only a hint;
/// callers with explicit stage information should set [OrbState] directly.
pub fn state_for_event(event: &AgentEvent) -> OrbState {
    match event {
        AgentEvent::RunStarted => OrbState::Working,
        AgentEvent::ProviderConnecting => OrbState::Connecting,
        AgentEvent::ProviderConnected => OrbState::Working,
        AgentEvent::ToolStarted { name } => state_for_tool(name),
        AgentEvent::ToolFinished => OrbState::Working,
        AgentEvent::ResponseStarted => OrbState::Composing,
        AgentEvent::ResponseFinished => OrbState::Idle,
        AgentEvent::AwaitingInput => OrbState::Listening,
        AgentEvent::Failed | AgentEvent::Cancelled => OrbState::Idle,
    }
}

/// Maps a tool name to a state using conservative keyword categories.
pub fn state_for_tool(name: &str) -> OrbState {
    let name = name.to_ascii_lowercase();
    if contains_any(
        &name,
        &[
            "search", "grep", "find", "lookup", "web", "browse", "retriev",
        ],
    ) {
        OrbState::Searching
    } else if contains_any(
        &name,
        &[
            "valid", "verify", "test", "check", "parse", "analy", "solve", "lint",
        ],
    ) {
        OrbState::Solving
    } else if contains_any(&name, &["pipeline", "orchestr", "batch", "weav"]) {
        OrbState::Weaving
    } else if contains_any(&name, &["format", "shape", "render", "schema", "serial"]) {
        OrbState::Shaping
    } else if contains_any(&name, &["write", "edit", "compose", "generate", "draft"]) {
        OrbState::Composing
    } else if contains_any(&name, &["connect", "socket", "http", "request"]) {
        OrbState::Connecting
    } else if contains_any(&name, &["listen", "input", "prompt"]) {
        OrbState::Listening
    } else if contains_any(&name, &["think", "breath"]) {
        OrbState::Breathing
    } else {
        OrbState::Working
    }
}

/// Applies an event to an animator.
pub fn handle_agent_event(animator: &mut OrbAnimator, event: &AgentEvent) {
    animator.set_state(state_for_event(event));
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
