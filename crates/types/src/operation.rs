use crate::{GroupId, LogId, NodeId, OpId, Seq, Size};
use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Body {
    GroupMessage { group_id: GroupId },
    DirectMessage { recipient: NodeId },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Header {
    pub author: NodeId,
    pub payload_size: Size,
    pub seq: Seq,
    pub log_id: LogId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Operation {
    pub id: OpId,
    pub header: Header,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct EncryptedOp(Operation);

impl EncryptedOp {
    /// This should only be used in contexts where decryption is actually allowed!
    /// The whole point is to pretend that the data remains encrypted in other contexts.
    pub fn decrypt(self, _key: &EncryptionKey) -> Operation {
        self.0
    }
}

/// Represents a private encryption key.
///
/// This is more of a reminder that a key is needed. This could be modeled better,
/// enforcing that the right key is used for the right [EncryptedOp].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncryptionKey;
