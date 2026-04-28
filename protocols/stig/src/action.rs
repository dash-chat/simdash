use derive_more::Display;
use named_id::RenameNone;

/// Actions for the Stig protocol.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, RenameNone)]
pub enum Action {
    /// Trigger a sync round
    TriggerSync,
}
