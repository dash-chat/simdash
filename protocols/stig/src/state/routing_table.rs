use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Announce, Metadata, Model, SourceId, StateVector};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerLogFact {
    pub state_vector: StateVector,
    pub age: Ticks,
    pub since_last_broadcast: Option<Ticks>,
    pub hops: Hops,
    pub relay: SourceId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingTable {
    pub facts: HashMap<TopicId, HashMap<SourceId, PeerLogFact>>,
}

impl RoutingTable {
    /// Record an op authored by this node.
    /// This causes the Announce for this topic to be broadcast at the next tick.
    pub fn record_local_topic_update(
        &mut self,
        model: &Model,
        topic_id: TopicId,
        item: Option<(LogId, LogHeight)>,
    ) {
        self.facts
            .entry(topic_id)
            .or_default()
            .entry(model.mesh_id)
            .and_modify(|f| f.state_vector = f.state_vector.merge(&StateVector::new(item)))
            .or_insert(PeerLogFact {
                state_vector: StateVector::new(item),
                age: 0.into(),
                since_last_broadcast: None,
                hops: 0.into(),
                relay: model.mesh_id,
            });
    }

    pub fn record_peer_announcement(&mut self, mut announce: Announce) {
        announce.hops += 1.into();
        for (topic_id, state_vector) in announce.topics {
            let entry = self
                .facts
                .entry(topic_id)
                .or_default()
                .entry(announce.author)
                .or_insert_with(|| PeerLogFact {
                    state_vector: StateVector::new(std::iter::empty()),
                    age: 0.into(),
                    since_last_broadcast: None,
                    hops: announce.hops,
                    relay: announce.relay,
                });

            // Always merge: retain the highest known height for each log.
            // NOTE: if a lower-hop relay arrives with a lower state vector, the merged
            // result will be optimistic (we'll think the peer has more data than the
            // low-hop route knows about). Acceptable for now.
            entry.state_vector = entry.state_vector.merge(&state_vector);
            entry.age = 0.into();

            // Only update the relay when this route has fewer hops.
            if announce.hops < entry.hops {
                entry.hops = announce.hops;
                entry.relay = announce.relay;
            }
        }
    }

    /// Given a set of incoming metadata, decide on the addressees for the payload to broadcast.
    //
    // OPTIMIZE: send different targets different payloads based on their state vectors
    pub fn targets<'a>(
        &self,
        model: &Model,
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
                        .filter(|(author, fact)| {
                            **author != model.mesh_id && !has_it(&fact.state_vector)
                        })
                        .map(|(author, _)| *author),
                );
            }
        }
        result
    }

    pub fn announcements_to_broadcast(&mut self, model: &Model) -> impl Iterator<Item = Announce> {
        let mut by_source: HashMap<SourceId, (Hops, SourceId, BTreeMap<TopicId, StateVector>)> =
            HashMap::new();
        for (topic_id, peers) in &mut self.facts {
            for (src, fact) in peers {
                // TODO: check that hops increases
                if fact
                    .since_last_broadcast
                    .map(|s| s >= model.sync_interval)
                    .unwrap_or(true)
                {
                    fact.since_last_broadcast = Some(0.into());
                    let entry = by_source
                        .entry(*src)
                        .or_insert_with(|| (fact.hops, fact.relay, BTreeMap::new()));
                    entry.2.insert(*topic_id, fact.state_vector.clone());
                }
            }
        }
        by_source
            .into_iter()
            .map(move |(author, (hops, _, topics))| Announce {
                author,
                relay: model.mesh_id,
                hops,
                topics,
            })
    }

    pub fn tick(&mut self) {
        for peers in self.facts.values_mut() {
            for fact in peers.values_mut() {
                fact.age += 1.into();
                fact.since_last_broadcast = fact.since_last_broadcast.map(|s| s + 1.into());
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

    fn relay_of(table: &RoutingTable, topic: u32, src: SourceId) -> SourceId {
        table.facts[&topic.into()][&src].relay
    }

    #[test]
    fn test_record_merge() {
        let alice = SourceId(NodeId(0).into());
        let relay1 = SourceId(NodeId(1).into());
        let relay2 = SourceId(NodeId(2).into());

        // state vectors are always unioned regardless of hop count
        let mut table = RoutingTable::default();
        let mut a1 = Announce::new(alice, statevec(0, [(1, 5), (2, 3)]));
        a1.hops = 1.into();
        a1.relay = relay1;
        let mut a2 = Announce::new(alice, statevec(0, [(1, 3), (2, 7)]));
        a2.hops = 2.into();
        a2.relay = relay2;

        table.record_peer_announcement(a1.clone());
        // relay1 stored (only route)
        assert_eq!(relay_of(&table, 0, alice), relay1);

        table.record_peer_announcement(a2.clone());
        // relay stays relay1 (lower hops); state vector is merged: log1=max(5,3)=5, log2=max(3,7)=7
        assert_eq!(relay_of(&table, 0, alice), relay1);
        let sv = &table.facts[&0.into()][&alice].state_vector;
        assert_eq!(sv[&1.into()], 5.into());
        assert_eq!(sv[&2.into()], 7.into());

        // reverse: higher-hop route arrives first, lower-hop route updates relay
        let mut table2 = RoutingTable::default();
        table2.record_peer_announcement(a2);
        assert_eq!(relay_of(&table2, 0, alice), relay2);
        table2.record_peer_announcement(a1);
        // relay updates to relay1 (fewer hops); state vector still merged
        assert_eq!(relay_of(&table2, 0, alice), relay1);
        let sv2 = &table2.facts[&0.into()][&alice].state_vector;
        assert_eq!(sv2[&1.into()], 5.into());
        assert_eq!(sv2[&2.into()], 7.into());
    }

    #[test]
    fn test_announcements_to_broadcast_sync_interval() {
        let me = SourceId::from(0);
        let alice = SourceId::from(1);
        let bobbi = SourceId::from(2);
        let sync_interval: Ticks = 3.into();

        let model = Model::new(me, sync_interval);
        let mut table = RoutingTable::default();

        // Record alice at t=0, bobbi at t=1
        table.record_peer_announcement(Announce::new(alice, statevec(0, [(1, 5)])));
        table.tick();
        table.record_peer_announcement(Announce::new(bobbi, statevec(0, [(2, 3)])));

        // Both should appear on the first call (since_last_broadcast is None for both)
        let announced: Vec<_> = table.announcements_to_broadcast(&model).collect();
        let authors: BTreeSet<SourceId> = announced.iter().map(|a| a.author).collect();
        assert_eq!(authors, BTreeSet::from([alice, bobbi]));

        // Immediately after, neither should appear (since_last_broadcast just reset to 0)
        let announced: Vec<_> = table.announcements_to_broadcast(&model).collect();
        assert!(announced.is_empty());

        // Tick twice more (since_last_broadcast reaches 2, still < 3)
        table.tick();
        table.tick();
        let announced: Vec<_> = table.announcements_to_broadcast(&model).collect();
        assert!(announced.is_empty());

        // One more tick brings since_last_broadcast to 3 == sync_interval; both should appear again
        table.tick();
        let announced: Vec<_> = table.announcements_to_broadcast(&model).collect();
        let authors: BTreeSet<SourceId> = announced.iter().map(|a| a.author).collect();
        assert_eq!(authors, BTreeSet::from([alice, bobbi]));

        // And again immediately after: nothing
        let announced: Vec<_> = table.announcements_to_broadcast(&model).collect();
        assert!(announced.is_empty());
    }

    #[test]
    fn test_targets() {
        let mut table = RoutingTable::default();

        let me = SourceId::from(0);
        let alice = SourceId::from(1);
        let bobbi = SourceId::from(2);
        let chuck = SourceId::from(3);

        let model = Model::new(me, 3.into());

        let a = Announce::new(alice, statevec(0, [(1, 3), (2, 3)]));
        let b = Announce::new(bobbi, statevec(0, [(1, 3), (2, 5)]));
        let c = Announce::new(chuck, statevec(0, [(1, 5), (2, 3)]));

        assert_eq!(
            table.targets(&model, &[meta(0, 1, 2), meta(0, 2, 4)]),
            btreeset!()
        );

        table.record_peer_announcement(a);

        assert_eq!(
            table.targets(&model, &[meta(0, 1, 2), meta(0, 2, 4)]),
            btreeset!()
        );

        table.record_peer_announcement(b);

        assert_eq!(table.targets(&model, &[meta(0, 1, 2)]), btreeset!());
        assert_eq!(
            table.targets(&model, &[meta(0, 1, 2), meta(0, 2, 4)]),
            btreeset!(alice)
        );

        table.record_peer_announcement(c);

        assert_eq!(
            table.targets(&model, &[meta(0, 1, 4)]),
            btreeset!(alice, bobbi)
        );
        assert_eq!(
            table.targets(&model, &[meta(0, 2, 4)]),
            btreeset!(alice, chuck)
        );
    }
}
