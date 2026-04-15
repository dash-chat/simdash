/// Generates `RoutingDispatchModel`, `RoutingDispatchState`, `AnyRoutingAction`, `AnyRoutingData`,
/// `AnyRoutingFx`, their `polestar::Machine` impl, and per-protocol helper functions,
/// dispatching across all listed protocols.
///
/// Each protocol is active simultaneously — `RoutingDispatchState` is a struct with one
/// field per protocol, and `AnyRoutingAction`/`AnyRoutingData`/`AnyRoutingFx` are
/// enums that name which protocol produced or receives the action/effect.
///
/// # Requirements
///
/// Each protocol's state type must have a `.tick()` method.
/// Each protocol's model type must implement `polestar::Machine` with:
/// - `State  = {state type}`
/// - `Action = RoutingAction<{action type}, {data type}>`
/// - `Fx     = Option<RoutingFx<{fx type}, {data type}>>`
/// - `Error  = anyhow::Error`
///
/// The Data type models datagrams flowing through the network.
///
/// # Example
///
/// ```rust,ignore
/// routing_dispatch! {
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
/// - `RoutingDispatchModel` struct with one field per protocol
/// - `RoutingDispatchState` struct with one field per protocol
/// - `AnyRoutingData`, `AnyRoutingAction`, `AnyRoutingFx` enums
/// - `polestar::Machine` impl on `RoutingDispatchModel`
/// - `RoutingDispatchModel::tick()` method
/// - `RoutingDispatchState::tick()` method

#[macro_export]
macro_rules! routing_dispatch {
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
        pub struct RoutingDispatchModel {
            $(
                #[doc = concat!("The model for the `", stringify!($field), "` protocol")]
                pub $field: $model_ty,
            )+
        }

        /// The state composed of all user-defined routing protocol states.
        #[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
        pub struct RoutingDispatchState {
            $(
                #[doc = concat!("The state for the `", stringify!($field), "` protocol")]
                pub $field: $state_ty,
            )+
        }

        impl GetRoutingOpInfo for RoutingDispatchState {
            fn routing_op_infos(&self) -> impl Iterator<Item = RoutingOpInfo> {
                let ops = std::iter::empty();
                $( let ops = ops.chain(GetRoutingOpInfo::routing_op_infos(&self.$field)); )+
                ops
            }
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

        impl Machine for RoutingDispatchModel {
            type State  = RoutingDispatchState;
            type Action = RoutingAction<AnyRoutingAction, AnyRoutingData>;
            type Fx     = Vec<RoutingFx<AnyRoutingFx, AnyRoutingData>>;
            type Error  = anyhow::Error;

            fn transition(
                &self,
                mut state: Self::State,
                action: Self::Action,
            ) -> polestar::TransitionResult<Self> {
                match action {
                    RoutingAction::Network(ProtocolNetworkAction::Tick) => {
                        {
                            $(
                                let (new, _fx) = self.$field.transition(state.$field, RoutingAction::tick())?;
                                state.$field = new;
                             )+
                        }
                        Ok((state, vec![]))
                    }

                    RoutingAction::Network(ProtocolNetworkAction::Receive { data }) => {
                        match data {
                            $(
                                AnyRoutingData::$Variant(data) => {
                                    let (new, fx) = self.$field.transition(
                                        state.$field,
                                        RoutingAction::Network(ProtocolNetworkAction::Receive {data}),
                                    )?;
                                    state.$field = new;
                                    let fx = fx.into_iter().map(|fx| crate::lift_fx(fx, AnyRoutingData::$Variant, AnyRoutingFx::$Variant)).collect();
                                    Ok((state, fx))
                                }
                            )+
                        }
                    }

                    RoutingAction::Protocol(action) => match action {
                        $(
                            AnyRoutingAction::$Variant(a) => {
                                let (new, fxes) = self.$field.transition(
                                    state.$field,
                                    RoutingAction::Protocol(a),
                                )?;
                                state.$field = new;
                                let result_fx = fxes.into_iter().map(|fx| {
                                    crate::lift_fx(fx, AnyRoutingData::$Variant, AnyRoutingFx::$Variant)
                                }).collect();
                                Ok((state, result_fx))
                            }
                        )+
                    },
                }
            }
        }
    };
}
