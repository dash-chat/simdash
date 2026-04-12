use crate::{Body, Header, NodeId, OpId, Operation, Size};

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
