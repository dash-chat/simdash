

The network actually should emit Protocol-specific effects.
And of course it needs to be driven by protocol-specific actions.

At the level of the network model itself, it only handles network-specific effects, and produces network-specific actions.
So the network model needs to extract network effects and produce network actions for the appropriate protocol.

The behavior needs to specify which protocol to use.
It must know at compile-time what all the protocols are.
So there is an enum for that:

```rust
/// macroized
enum RoutingAction {
    Flooding(FloodingAction),
    // ...
}

/// macroized
enum RoutingData {
    Flooding(FloodingData),
    // ...
}
```

Each layer looks different:

- Behavior
- Network
- Node
- Routing
- Protocol

The routing protocols are defined independently, with no knowledge of the model components sitting above them, other than the common Action type of `Receive` and `Tick`, and the common Effect type of `Send`.

## Behavior-Network

```rust
pub enum NetworkAction {
    Tick,
    EvolveTopology,
    Node(NodeId, NodeAction),
}

/// Doesn't include Receive
pub enum NodeAction {
    Chat(ChatAction),
    Protocol(RoutingAction),
}

pub enum ChatAction {
    ...
}

// ???
pub enum NetworkEffect {
    Routing(RoutingEffect),
}
```

# Action interfaces 

## Network-Node

```rust
/// Receive gets added in when passing NodeAction through to the Node model
pub enum NetworkNodeAction {
    Receive { from: NodeId, data: RoutingData },
    Chat(ChatAction),
    Protocol(RoutingAction),
}

```

## Node-Routing

```rust
/// Chat gets removed
pub enum ProtocolAction {
    Receive { from: NodeId, data: RoutingData },
    Protocol(RoutingAction),
}

```

## Routing-Protocol

The protocol model itself receives `ProtocolAction` but with the data and action types replaced with the protocol-specific types.

The routing model mainly dispatches based on RoutingData or RoutingAction so that the protocol model can be called with its specific types.


```rust
pub enum RoutingFloodingAction {
    Receive { from: NodeId, data: FloodingData },
    Flooding(FloodingAction),
}

fn transition(state: RoutingDispatchState, action: ProtocolAction) -> (RoutingDispatchState, RoutingFx) {
    match action {
        ProtocolAction::Receive { from, data } => {
            match data {
                RoutingDataDispatch::Flooding(data) => {
                    let (protocol, fx) = self.flooding.transition(state.flooding, action)?;
                }
            }
        }
        ProtocolAction::Protocol(action) => {
            match action {
                RoutingActionDispatch::Flooding(action) => {
                    let (protocol, fx) = self.flooding.transition(state.flooding, action)?;
                }
            }
        }
    }
}
```

# Effect interfaces

Now on the way back up...

## Protocol-Routing

The protocol emits its own effect:

```rust

pub enum FloodingProtocolEffect {
    Send(SendEffect<FloodingData>),
    Protocol(FloodingEffect),
}
```

## Routing-Node

```rust

pub enum RoutingNodeEffect {
    Send(SendEffect<RoutingData>),
    Protocol(RoutingEffect), // macroized
}
```

## Node-Network

Possible Chat effects get added in here, though this may not be modeled.

```rust
pub type NodeNetworkEffect {
    Chat(ChatEffect),
    Send(SendEffect),
    Protocol(RoutingEffect),
}
```

## Network-Behavior

The network will handle any Send effect, since that is the main point of the network.

The only possible effect emitted is the protocol-specific effect, and chat effects if we do that.

```rust
pub enum NetworkEffect {
    Routing(RoutingEffect),
    Chat(ChatEffect),
}
```