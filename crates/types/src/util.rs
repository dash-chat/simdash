use named_id::Nameable;

use crate::{Body, Header, NodeId, OpId, Operation, Seq, Size, TopicId};

pub fn basic_op(id: impl Into<OpId>) -> Operation {
    let id = id.into();
    Operation {
        id,
        header: Header {
            author: NodeId::from(1),
            payload_size: Size::from(100u64),
            seq: Seq::from(2),
            topic_id: TopicId::from(4),
        },
        body: Body::DirectMessage {
            recipient: NodeId::from(2),
        },
    }
}
