mod behavior;
mod ids;
mod operation;
pub use behavior::*;
pub use ids::*;
pub use operation::*;

pub use named_id::{Rename, RenameAll, RenameNone};

pub mod util;

use derive_more::{Add, Deref, Display, From};
use serde::{Deserialize, Serialize};

pub trait ExtractMetadata<T> {
    fn extract_metadata(&self) -> T;
}

pub trait GetRoutingOpInfo {
    fn routing_op_infos(&self) -> impl Iterator<Item = RoutingOpInfo>;
}
