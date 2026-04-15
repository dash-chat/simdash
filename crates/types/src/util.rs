use crate::{Body, Header, LogId, NodeId, OpId, Operation, Seq, Size};

pub fn basic_op(id: impl Into<OpId>) -> Operation {
    Operation {
        id: id.into(),
        header: Header {
            author: NodeId::from(1),
            payload_size: Size::from(100u64),
            seq: Seq::from(2),
            log_id: LogId::from(3),
        },
        body: Body::DirectMessage {
            recipient: NodeId::from(2),
        },
    }
}
