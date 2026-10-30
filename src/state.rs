/// The visual state of an orb.
///
/// Idle is the resting state. The other variants correspond to the nine
/// animation algorithms described in the project specification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OrbState {
    /// The agent is ready and no request is active.
    #[default]
    Idle,
    /// The agent is working without a more specific signal.
    Working,
    /// A search or lookup is in progress.
    Searching,
    /// Results are being validated, parsed, or solved.
    Solving,
    /// The agent is waiting for user input.
    Listening,
    /// A provider connection is being established or retried.
    Connecting,
    /// Several related tool stages are running together.
    Weaving,
    /// A response is being generated.
    Composing,
    /// A calm, generic thinking indicator.
    Breathing,
    /// Structured output is being shaped or finalized.
    Shaping,
}

impl OrbState {
    /// Every state, in display order.
    pub const ALL: [Self; 10] = [
        Self::Idle,
        Self::Working,
        Self::Searching,
        Self::Solving,
        Self::Listening,
        Self::Connecting,
        Self::Weaving,
        Self::Composing,
        Self::Breathing,
        Self::Shaping,
    ];

    /// A short, stable, human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Searching => "Searching",
            Self::Solving => "Solving",
            Self::Listening => "Listening",
            Self::Connecting => "Connecting",
            Self::Weaving => "Weaving",
            Self::Composing => "Composing",
            Self::Breathing => "Breathing",
            Self::Shaping => "Shaping",
        }
    }
}
