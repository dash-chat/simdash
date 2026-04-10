/// Generates `RoutingModel`, `RoutingState`, `RoutingAction`, `RoutingFx`,
/// and their `polestar::Machine` impl, dispatching across all listed protocols.
///
/// Each protocol is active simultaneously — `RoutingState` is a struct with one
/// field per protocol, and `RoutingAction`/`RoutingFx` are enums that name which
/// protocol produced or receives the action/effect.
///
/// # Requirements
///
/// Each protocol's state type must implement [`simdash_state::TickableState`].
/// Each protocol's model type must implement `polestar::Machine` with:
/// - `State  = <state type>`
/// - `Action = <action type>`
/// - `Fx     = Option<<fx type>>`
/// - `Error  = anyhow::Error`
///
/// Additionally, each protocol provides a Data type which is used to model
/// datagrams flowing through the network.
/// The `RoutingAction::ReceiveData` models incoming data,
/// and the `RoutingFx::UnicastData` and `RoutingFx::BroadcastData` model outgoing data.
///
/// # Example
///
/// ```rust,ignore
/// routing_protocols! {
///     Flooding / flooding => {
///         model:  my_crate::routing::flooding::FloodingModel,
///         state:  my_crate::routing::flooding::FloodingState,
///         action: my_crate::routing::flooding::FloodingAction,
///         fx:     my_crate::routing::flooding::FloodingFx,
///         data:   my_crate::routing::flooding::FloodingData,
///     },
/// }
/// ```
///
/// Expands to structs `RoutingModel`, `RoutingState`, enum `RoutingAction`,
/// enum `RoutingFx`, method `RoutingState::tick()`, and a `polestar::Machine`
/// impl on `RoutingModel` that dispatches across all listed protocols.
#[macro_export]
macro_rules! routing_protocols {
    (
        $(
            $Variant:ident / $field:ident => {
                model:  $model_ty:ty,
                state:  $state_ty:ty,
                action: $action_ty:ty,
                fx:     $fx_ty:ty,
                data:   $data_ty:ty,
            }
        ),+ $(,)?
    ) => {
        pub struct RoutingModel {
            $( pub $field: $model_ty, )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
        pub struct RoutingState {
            $( pub $field: $state_ty, )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingAction {
            ReceiveData {
                from: NodeId,
                data: RoutingData,
            },
            Protocol(RoutingProtocolAction),
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingData {
            $( $Variant($data_ty), )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingProtocolAction {
            $( $Variant($action_ty), )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingFx {
            Network(RoutingNetworkFx),
            Node(RoutingNodeFx),
            // Protocol(RoutingProtocolFx),
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingNodeFx {
            StoreOps { ops: Vec<simdash_state::chat::Operation> }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RoutingNetworkFx {
            UnicastData {
                to: simdash_ids::NodeId,
                data: RoutingData,
            },
            BroadcastData {
                data: RoutingData,
            }
        }

        // #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        // pub enum RoutingProtocolFx {
        //     $( $Variant(Option<$fx_ty>), )+
        // }

        impl RoutingState {
            pub fn tick(&mut self) {
                $( <$state_ty as simdash_state::TickableState>::tick(&mut self.$field); )+
            }
        }

        impl polestar::Machine for RoutingModel {
            type State  = RoutingState;
            type Action = RoutingAction;
            type Fx     = RoutingFx;
            type Error  = anyhow::Error;

            fn transition(
                &self,
                mut state: Self::State,
                action: Self::Action,
            ) -> polestar::TransitionResult<Self> {
                match action {
                    $(
                        RoutingAction::$Variant(a) => {
                            let (s, fx) = self.$field.transition(state.$field, a)?;
                            state.$field = s;
                            Ok((state, RoutingFx::$Variant(fx)))
                        }
                    )+
                }
            }
        }
    };
}
