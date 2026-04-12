use crate::chat::{Body, Header, Operation};
use simdash_ids::{NodeId, OpId, Size};

pub fn basic_op(id: impl Into<OpId>) -> Operation {
    Operation {
        id: id.into(),
        header: Header {
            author: NodeId::from(1),
            payload_size: Size::from(100u64),
        },
        body: Body::DirectMessage {
            recipient: NodeId::from(2),
        },
    }
}
