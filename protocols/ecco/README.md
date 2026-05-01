# ecco routing protocol




## 1: Announcement Origination

#### `SourceId`

This is a topic-based routing protocol, which means that many nodes in the network may be interested in the same data.
When relay nodes receive announcements, they will be receiving many announcements about the same topic from their peers.
Some of these duplicates will be due to copies of an announcement from a single source traveling through different paths, and these should be deduplicated.
Some of the duplicates will be due to actually different announcements from different sources, and these need to be kept distinct.
So, the purpose of the Source ID is to ensure that at least one announcement per source is recorded by each relay, so that data can be relayed along the appropriate routes to reach those sources.

Hence, SourceID only needs to be stable for the duration of a roundtrip, from broadcasting an announcement to receiving corresponding data.
Different SourceIDs can be used for different topic announcements from the same node, and the node can use a different SourceID for subsequent announcements of the same topic.
The conditions and frequency of SourceID rotation is a parameter to study.

### Announcements

Each node periodically broadcasts topics with state vectors in an Announcement message.
This period's interval is a parameter.


## 2: Announcement Relaying


Each node stores all announcements it receives from other nodes, and does its best to relay them to any other node it encounters at least once.
(Ideally, announcements would be relayed *exactly* once to each peer.
In unicast, the list of nodes relayed to can be stored to prevent duplicates.
In broadcast, the list of "who was around when I sent this" can only be approximated.)
Before relaying, the hop count of the stored announcement is incremented, and the relay ID is updated.

Announcements may arrive by multiple routes, so nodes only store the announcement(s) with the lowest hop count for a given SourceId.
In broadcast networks, nodes can expect the same Announcement they sent to arrive back with a hop count incremented by 1, which they can discard.
This can be used as an acknowledgement of delivery.

If an announcement results in a new route being established, and there is a relevant payload, this will trigger the payload being sent along that route.
The payload must be addressed to this node (must look identical to if the announcement and the payload came in reverse order.)


## 3: Announcement Destination


When a node receives an Announcement for a topic it's subscribed to, it can now send a Payload with the diff.
The Payload will be addressed to the relay by which the Announcement was received.


## 4: Payload Origination

See 3.

## 5: Payload Relaying

A payload is always addressed to some set of nodes.
A node receiving a payload will always store it, but only re-broadcast it if:
1. its SourceId is in the set of addressees
2. there is a route established by prior received Announcements

TODO: what if a route is established after having stored it?


## 6: Payload Destination


When a node receives a Payload for a topic it's subscribed to, it will still perform any relaying based on other Announcements it's received.
Then it will store the data, which updates its state vector for future Announcements.


