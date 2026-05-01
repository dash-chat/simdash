use derive_more::Display;
use named_id::RenameNone;

/// Actions for the Noop protocol.
#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, RenameNone)]
pub enum NoopAction {
    /// A no-op action
    Noop,
}
