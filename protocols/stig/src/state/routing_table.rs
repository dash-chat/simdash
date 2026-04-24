use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Announce, Metadata, SourceId, StateVector};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerLogFact {
    pub state_vector: StateVector,
    pub age: Ticks,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingTable {
    facts: HashMap<TopicId, HashMap<SourceId, PeerLogFact>>,
}

impl RoutingTable {
    pub fn record(&mut self, announce: Announce) {
        for (topic_id, state_vector) in announce.topics {
            let entry = self
                .facts
                .entry(topic_id)
                .or_default()
                .entry(announce.author)
                .or_insert_with(|| PeerLogFact {
                    state_vector: state_vector.clone(),
                    age: 0.into(),
                });
            entry.age = 0.into();
            entry.state_vector = state_vector;
        }
    }

    /// Given a set of incoming metadata, decide on the addressees for the payload to broadcast.
    //
    // OPTIMIZE: send different targets different payloads based on their state vectors
    pub fn targets<'a>(
        &self,
        metadata: impl IntoIterator<Item = &'a Metadata>,
    ) -> BTreeSet<SourceId> {
        let mut result = BTreeSet::new();
        for meta in metadata {
            let Some(topic_peers) = self.facts.get(&meta.topic_id) else {
                continue;
            };
            let threshold = LogHeight::from_seq(meta.seq);
            let has_it = |v: &StateVector| {
                v.get(&meta.log_id)
                    .map(|h| *h >= threshold)
                    .unwrap_or(false)
            };
            let any_has_it = topic_peers.values().any(|fact| has_it(&fact.state_vector));
            if any_has_it {
                result.extend(
                    topic_peers
                        .iter()
                        .filter(|(_, fact)| !has_it(&fact.state_vector))
                        .map(|(src, _)| *src),
                );
            }
        }
        result
    }

    pub fn tick(&mut self) {
        for peers in self.facts.values_mut() {
            for fact in peers.values_mut() {
                fact.age += 1.into();
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
