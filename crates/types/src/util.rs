use std::{
    collections::HashMap,
    sync::LazyLock,
    sync::{Mutex, atomic::AtomicU32},
};

use crate::{Body, Header, NodeId, OpId, Operation, Seq, Size, TopicId};

static NUM_OPS: AtomicU32 = AtomicU32::new(0);
static SEQS: LazyLock<Mutex<HashMap<(NodeId, TopicId), Seq>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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

pub fn group_op(author: impl Into<NodeId>, topic_id: impl Into<TopicId>) -> Operation {
    let id = NUM_OPS
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .into();
    let author = author.into();
    let topic_id = topic_id.into();

    let mut seqmap = SEQS.lock().unwrap();
    let seq_mut = seqmap.entry((author, topic_id)).or_insert(Seq::from(0));
    let seq = *seq_mut;

    *seq_mut += 1.into();

    Operation {
        id,
        header: Header {
            author,
            payload_size: Size::from(100u64),
            seq,
            topic_id,
        },
        body: Body::GroupMessage { group_id: topic_id },
    }
}
