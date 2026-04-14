/// Generates `RoutingModel`, `RoutingState`, `AnyRoutingAction`, `AnyRoutingData`,
/// `AnyRoutingFx`, their `polestar::Machine` impl, and per-protocol helper functions,
/// dispatching across all listed protocols.
///
/// Each protocol is active simultaneously — `RoutingState` is a struct with one
/// field per protocol, and `AnyRoutingAction`/`AnyRoutingData`/`AnyRoutingFx` are
/// enums that name which protocol produced or receives the action/effect.
///
/// # Requirements
///
/// Each protocol's state type must have a `.tick()` method.
/// Each protocol's model type must implement `polestar::Machine` with:
/// - `State  = {state type}`
/// - `Action = ProtocolAction<{action type}, {data type}>`
/// - `Fx     = Option<ProtocolFx<{fx type}, {data type}>>`
/// - `Error  = anyhow::Error`
///
/// The Data type models datagrams flowing through the network.
///
/// # Example
///
/// ```rust,ignore
/// routing_protocols! {
///     Flooding / flooding => {
///         model:  FloodingModel,
///         state:  FloodingState,
///         action: FloodingAction,
///         fx:     FloodingFx,
///         data:   FloodingWireData,
///     },
/// }
/// ```
///
/// Expands to:
/// - `RoutingModel` struct with one field per protocol
/// - `RoutingState` struct with one field per protocol
/// - `AnyRoutingData`, `AnyRoutingAction`, `AnyRoutingFx` enums
/// - `polestar::Machine` impl on `RoutingModel`
/// - `RoutingModel::tick()` method
/// - `RoutingState::tick()` method

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

        /// The model composed of all user-defined routing protocols.
        pub struct RoutingModel {
            $(
                #[doc = concat!("The model for the `", stringify!($field), "` protocol")]
                pub $field: $model_ty,
            )+
        }

        impl RoutingModel {
            /// Advance the routing model by one tick
            pub fn tick(&self, state: RoutingState) -> polestar::TransitionResult<Self> {
                self.transition(state, ProtocolAction::Network(ProtocolNetworkAction::Tick))
            }
        }


        /// The state composed of all user-defined routing protocol states.
        #[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
        pub struct RoutingState {
            $(
                #[doc = concat!("The state for the `", stringify!($field), "` protocol")]
                pub $field: $state_ty,
            )+
        }

        /// An enum that unifies all possible routing wire data types.
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AnyRoutingData {
            $(
                #[doc = concat!("The wire data type for the `", stringify!($Variant), "` protocol")]
                $Variant($data_ty),
            )+
        }

        /// An enum that unifies all possible routing action types.
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AnyRoutingAction {
            $(
                #[doc = concat!("The action type for the `", stringify!($Variant), "` protocol")]
                $Variant($action_ty),
            )+
        }

        /// An enum that unifies all possible routing effect types.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum AnyRoutingFx {
            $(
                #[doc = concat!("The effect type for the `", stringify!($Variant), "` protocol")]
                $Variant($fx_ty),
            )+
        }

        impl Tickable for RoutingState {
            fn tick(&mut self) {
                $( Tickable::tick(&mut self.$field); )+
            }
        }

        impl Machine for RoutingModel {
            type State  = RoutingState;
            type Action = ProtocolAction<AnyRoutingAction, AnyRoutingData>;
            type Fx     = Option<ProtocolFx<AnyRoutingFx, AnyRoutingData>>;
            type Error  = anyhow::Error;

            fn transition(
                &self,
                mut state: Self::State,
                action: Self::Action,
            ) -> polestar::TransitionResult<Self> {
                match action {
                    ProtocolAction::Network(ProtocolNetworkAction::Tick) => {
                        Tickable::tick(&mut state);
                        Ok((state, None))
                    }

                    ProtocolAction::Network(ProtocolNetworkAction::Receive { from, data }) => {
                        let mut result_fx = None;
                        $(
                            let protocol_data: std::collections::HashSet<$data_ty> = data
                                .iter()
                                .filter_map(|d| match d {
                                    AnyRoutingData::$Variant(inner) => Some(inner.clone()),
                                    #[allow(unreachable_patterns)]
                                    _ => None,
                                })
                                .collect();
                            if !protocol_data.is_empty() {
                                let (new, fx) = self.$field.transition(
                                    state.$field,
                                    ProtocolAction::Network(ProtocolNetworkAction::Receive {
                                        from,
                                        data: protocol_data,
                                    }),
                                )?;
                                state.$field = new;
                                result_fx = fx.map(|fx| {
                                    crate::lift_fx(fx, AnyRoutingData::$Variant, AnyRoutingFx::$Variant)
                                });
                            }
                        )+
                        Ok((state, result_fx))
                    }

                    ProtocolAction::Protocol(action) => match action {
                        $(
                            AnyRoutingAction::$Variant(a) => {
                                let (new, fx) = self.$field.transition(
                                    state.$field,
                                    ProtocolAction::Protocol(a),
                                )?;
                                state.$field = new;
                                Ok((state, fx.map(|fx| {
                                    crate::lift_fx(fx, AnyRoutingData::$Variant, AnyRoutingFx::$Variant)
                                })))
                            }
                        )+
                    },
                }
            }
        }
    };
}
