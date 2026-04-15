/// Actions for the Noop protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// Trigger a sync round
    TriggerSync,
}
