use named_id::{Nameable, Rename};
use simdash_types::{LogId, Seq, TopicId};

use crate::Op;

pub fn make_stig_op(topic_id: TopicId, log_id: LogId, seq: Seq) -> Op {
    use crate::Metadata;
    use simdash_types::{EncryptedOp, util::basic_op};

    let name = format!(
        "{}.{}.{}",
        topic_id.renamed(),
        log_id.renamed(),
        seq.renamed()
    );
    let op = basic_op(0);
    Op {
        metadata: Metadata {
            topic_id,
            log_id,
            seq,
        },
        encrypted: EncryptedOp::from(op),
    }
    .with_name(&name)
}
