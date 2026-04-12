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
/// - `State  = <state type>`
/// - `Action = ProtocolAction<<action type>, <data type>>`
/// - `Fx     = Option<ProtocolFx<<fx type>, <data type>>>`
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
/// - Per-protocol helpers: `routing_{field}_action`, `routing_{field}_protocol_action`,
///   `routing_{field}_protocol_fx`

#[macro_export]
macro_rules! routing_protocols {
    (
        $(
            $field:ident => {
                model:  $model_ty:ty,
                state:  $state_ty:ty,
                action: $action_ty:ty,
                fx:     $fx_ty:ty,
                data:   $data_ty:ty,
            }
        ),+ $(,)?
    ) => {
        ::paste::paste! {
            routing_protocols! {
                $(
                    [ <$field:camel> ] / $field => {
                        model:  $model_ty,
                        state:  $state_ty,
                        action: $action_ty,
                        fx:     $fx_ty,
                        data:   $data_ty,
                    }
                )+
            }
        }
    };

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

        use polestar::Machine;

        pub struct RoutingModel {
            $( pub $field: $model_ty, )+
        }

        impl RoutingModel {
            pub fn tick(&self, state: RoutingState) -> polestar::TransitionResult<Self> {
                self.transition(
                    state,
                    simdash_routing_common::ProtocolAction::Network(
                        simdash_routing_common::ProtocolNetworkAction::Tick,
                    ),
                )
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
        pub struct RoutingState {
            $( pub $field: $state_ty, )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AnyRoutingData {
            $( $Variant($data_ty), )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AnyRoutingAction {
            $( $Variant($action_ty), )+
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum AnyRoutingFx {
            $( $Variant($fx_ty), )+
        }

        impl RoutingState {
            pub fn tick(&mut self) {
                $( self.$field.tick(); )+
            }
        }

        impl polestar::Machine for RoutingModel {
            type State  = RoutingState;
            type Action = simdash_routing_common::ProtocolAction<AnyRoutingAction, AnyRoutingData>;
            type Fx     = Option<simdash_routing_common::ProtocolFx<AnyRoutingFx, AnyRoutingData>>;
            type Error  = anyhow::Error;

            fn transition(
                &self,
                mut state: Self::State,
                action: Self::Action,
            ) -> polestar::TransitionResult<Self> {
                match action {
                    simdash_routing_common::ProtocolAction::Network(
                        simdash_routing_common::ProtocolNetworkAction::Tick,
                    ) => {
                        state.tick();
                        Ok((state, None))
                    }

                    simdash_routing_common::ProtocolAction::Network(
                        simdash_routing_common::ProtocolNetworkAction::Receive { from, data },
                    ) => {
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
                                    simdash_routing_common::ProtocolAction::Network(
                                        simdash_routing_common::ProtocolNetworkAction::Receive {
                                            from,
                                            data: protocol_data,
                                        },
                                    ),
                                )?;
                                state.$field = new;
                                result_fx = fx.map(|fx| match fx {
                                    simdash_routing_common::ProtocolFx::Network(fx) => match fx {
                                        simdash_routing_common::SendFx::BroadcastData { data } => {
                                            simdash_routing_common::ProtocolFx::Network(
                                                simdash_routing_common::SendFx::BroadcastData {
                                                    data: data.into_iter().map(AnyRoutingData::$Variant).collect(),
                                                },
                                            )
                                        }
                                        simdash_routing_common::SendFx::UnicastData { to, data } => {
                                            simdash_routing_common::ProtocolFx::Network(
                                                simdash_routing_common::SendFx::UnicastData {
                                                    to,
                                                    data: data.into_iter().map(AnyRoutingData::$Variant).collect(),
                                                },
                                            )
                                        }
                                    },
                                    simdash_routing_common::ProtocolFx::Node(fx) => {
                                        simdash_routing_common::ProtocolFx::Node(fx)
                                    }
                                    simdash_routing_common::ProtocolFx::Protocol(fx) => {
                                        simdash_routing_common::ProtocolFx::Protocol(AnyRoutingFx::$Variant(fx))
                                    }
                                });
                            }
                        )+
                        Ok((state, result_fx))
                    }

                    simdash_routing_common::ProtocolAction::Protocol(action) => match action {
                        $(
                            AnyRoutingAction::$Variant(a) => {
                                let (new, fx) = self.$field.transition(
                                    state.$field,
                                    simdash_routing_common::ProtocolAction::Protocol(a),
                                )?;
                                state.$field = new;
                                Ok((state, fx.map(|fx| match fx {
                                    simdash_routing_common::ProtocolFx::Network(fx) => match fx {
                                        simdash_routing_common::SendFx::BroadcastData { data } => {
                                            simdash_routing_common::ProtocolFx::Network(
                                                simdash_routing_common::SendFx::BroadcastData {
                                                    data: data.into_iter().map(AnyRoutingData::$Variant).collect(),
                                                },
                                            )
                                        }
                                        simdash_routing_common::SendFx::UnicastData { to, data } => {
                                            simdash_routing_common::ProtocolFx::Network(
                                                simdash_routing_common::SendFx::UnicastData {
                                                    to,
                                                    data: data.into_iter().map(AnyRoutingData::$Variant).collect(),
                                                },
                                            )
                                        }
                                    },
                                    simdash_routing_common::ProtocolFx::Node(fx) => {
                                        simdash_routing_common::ProtocolFx::Node(fx)
                                    }
                                    simdash_routing_common::ProtocolFx::Protocol(fx) => {
                                        simdash_routing_common::ProtocolFx::Protocol(AnyRoutingFx::$Variant(fx))
                                    }
                                })))
                            }
                        )+
                    },
                }
            }
        }

        // Per-protocol helper functions are defined using paste! for identifier concatenation.
        // These require `paste` to be available in scope at the call site:
        //   use ::paste::paste;
        $(
            ::paste::paste! {
                /// Lift a protocol-specific action into a `RoutingAction`.
                pub fn [< routing_ $field _action >](action: $action_ty) -> RoutingAction {
                    RoutingAction::Protocol(AnyRoutingAction::$Variant(action))
                }

                /// Lift a protocol-specific `ProtocolAction` into a `RoutingAction`.
                pub fn [< routing_ $field _protocol_action >](
                    action: simdash_routing_common::ProtocolAction<$action_ty, $data_ty>,
                ) -> RoutingAction {
                    match action {
                        simdash_routing_common::ProtocolAction::Network(network_action) => {
                            match network_action {
                                simdash_routing_common::ProtocolNetworkAction::Tick => {
                                    RoutingAction::Network(
                                        simdash_routing_common::ProtocolNetworkAction::Tick,
                                    )
                                }
                                simdash_routing_common::ProtocolNetworkAction::Receive {
                                    from,
                                    data,
                                } => RoutingAction::Network(
                                    simdash_routing_common::ProtocolNetworkAction::Receive {
                                        from,
                                        data: data
                                            .into_iter()
                                            .map(AnyRoutingData::$Variant)
                                            .collect(),
                                    },
                                ),
                            }
                        }
                        simdash_routing_common::ProtocolAction::Protocol(action) => {
                            RoutingAction::Protocol(AnyRoutingAction::$Variant(action))
                        }
                    }
                }

                /// Lift a protocol-specific `ProtocolFx` into a `RoutingFx`.
                pub fn [< routing_ $field _protocol_fx >](
                    fx: simdash_routing_common::ProtocolFx<$fx_ty, $data_ty>,
                ) -> RoutingFx {
                    match fx {
                        simdash_routing_common::ProtocolFx::Network(fx) => match fx {
                            simdash_routing_common::SendFx::BroadcastData { data } => {
                                RoutingFx::Network(simdash_routing_common::SendFx::BroadcastData {
                                    data: data
                                        .into_iter()
                                        .map(AnyRoutingData::$Variant)
                                        .collect(),
                                })
                            }
                            simdash_routing_common::SendFx::UnicastData { to, data } => {
                                RoutingFx::Network(simdash_routing_common::SendFx::UnicastData {
                                    to,
                                    data: data
                                        .into_iter()
                                        .map(AnyRoutingData::$Variant)
                                        .collect(),
                                })
                            }
                        },
                        simdash_routing_common::ProtocolFx::Node(fx) => {
                            RoutingFx::Node(fx)
                        }
                        simdash_routing_common::ProtocolFx::Protocol(fx) => {
                            RoutingFx::Protocol(AnyRoutingFx::$Variant(fx))
                        }
                    }
                }
            }
        )+
    };
}
