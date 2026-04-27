use named_id::RenameNone;

/// Actions for the Stig protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, RenameNone)]
pub enum Action {
    /// Trigger a sync round
    TriggerSync,
}
