use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Announce, Metadata, SourceId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerLogFact {
    pub log_id: LogId,
    pub log_height: LogHeight,
    pub age: Ticks,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingTable {
    peers: HashMap<TopicId, HashMap<SourceId, PeerLogFact>>,
}

impl RoutingTable {
    pub fn record(&mut self, announce: Announce) {
        todo!();
        // self.peers
        //     .entry(topic_id)
        //     .or_insert_with(HashMap::new)
        //     .insert(
        //         mesh_id,
        //         PeerLogFact {
        //             log_id,
        //             log_height,
        //             age: 0.into(),
        //         },
        //     );
    }

    /// Given a set of incoming metadata, decide on the addressees for the payload to broadcast.
    //
    // OPTIMIZE: send different targets different payloads based on their state vectors
    pub fn targets<'a>(
        &self,
        metadata: impl IntoIterator<Item = &'a Metadata>,
    ) -> BTreeSet<SourceId> {
        todo!();
    }

    pub fn tick(&mut self) {
        for peers in self.peers.values_mut() {
            for peer in peers.values_mut() {
                peer.age += 1.into();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use maplit::btreeset;

    use crate::StateVector;

    use super::*;

    fn statevec(
        topic: impl Into<TopicId>,
        items: impl IntoIterator<Item = (u32, u32)>,
    ) -> BTreeMap<TopicId, StateVector> {
        BTreeMap::from([(
            topic.into(),
            StateVector::new(
                items
                    .into_iter()
                    .map(|(log_id, log_height)| (log_id.into(), log_height.into())),
            ),
        )])
    }

    fn meta(topic: impl Into<TopicId>, log_id: impl Into<LogId>, seq: impl Into<Seq>) -> Metadata {
        Metadata {
            topic_id: topic.into(),
            log_id: log_id.into(),
            seq: seq.into(),
        }
    }

    #[test]
    fn test_record() {
        let mut table = RoutingTable::default();

        let alice = SourceId::new();
        let bobbi = SourceId::new();
        let chuck = SourceId::new();

        let a = Announce::new(alice, statevec(0, [(1, 3), (2, 3)]));
        let b = Announce::new(bobbi, statevec(0, [(1, 3), (2, 5)]));
        let c = Announce::new(chuck, statevec(0, [(1, 5), (2, 3)]));

        assert_eq!(table.targets(&[meta(0, 1, 2), meta(0, 2, 4)]), btreeset!());

        table.record(a);

        assert_eq!(table.targets(&[meta(0, 1, 2), meta(0, 2, 4)]), btreeset!());

        table.record(b);

        assert_eq!(table.targets(&[meta(0, 1, 2)]), btreeset!());
        assert_eq!(
            table.targets(&[meta(0, 1, 2), meta(0, 2, 4)]),
            btreeset!(alice)
        );

        table.record(c);

        assert_eq!(table.targets(&[meta(0, 1, 4)]), btreeset!(alice, bobbi));
        assert_eq!(table.targets(&[meta(0, 2, 4)]), btreeset!(alice, chuck));
    }
}
