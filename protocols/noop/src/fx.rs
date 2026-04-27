use named_id::RenameNone;

/// Effects specific to the Noop protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash, RenameNone)]
#[must_use]
pub enum NoopFx {}
