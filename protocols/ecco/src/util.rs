use named_id::{Nameable, Rename};
use simdash_types::{NodeId, TopicId};

use crate::Op;

pub fn make_ecco_op(author: NodeId, topic_id: TopicId) -> Op {
    use crate::Metadata;
    use simdash_types::{EncryptedOp, util::group_op};

    let op = group_op(author, topic_id);

    let op = Op {
        metadata: Metadata::from_op(&op),
        encrypted: EncryptedOp::from(op),
    };
    let name = format!(
        "{}.{}.{}",
        topic_id.renamed(),
        author.renamed(),
        op.metadata.seq.renamed()
    );

    op.with_name(&name)
}
