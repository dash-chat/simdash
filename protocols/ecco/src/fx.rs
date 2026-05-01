use derive_more::Display;
use named_id::RenameNone;

/// Effects specific to the Ecco protocol
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, RenameNone)]
#[must_use]
pub enum Fx {}
