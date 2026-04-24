use crate::{GroupId, LogId, NodeId, OpId, Seq, Size, TopicId};
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
    pub topic_id: TopicId,
    // pub log_id: LogId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Operation {
    pub id: OpId,
    pub header: Header,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct EncryptedOp(Operation);

impl std::fmt::Debug for EncryptedOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedOp")
    }
}

impl EncryptedOp {
    /// This should only be used in contexts where decryption is actually allowed!
    /// The whole point is to pretend that the data remains encrypted in other contexts.
    pub fn decrypt(self, _key: &EncryptionKey) -> Operation {
        self.0
    }

    /// Reveal the OpId of the encrypted op.
    /// Model logic should never make use of this, but it's useful
    /// for external observability
    pub fn op_id(&self) -> OpId {
        self.0.id
    }

    /// Size of the op.
    /// TODO: not accurate, doesn't include header
    pub fn size(&self) -> Size {
        self.0.header.payload_size
    }
}

/// Represents a private encryption key.
///
/// This is more of a reminder that a key is needed. This could be modeled better,
/// enforcing that the right key is used for the right [EncryptedOp].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncryptionKey;

/// Used by the simulation runner's visualizations
#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoutingOpInfo {
    pub id: OpId,
    pub size: Size,
    // pub since_last_send: Option<u32>,
}
