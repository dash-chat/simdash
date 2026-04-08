use simdash_action::routing::flooding::FloodingAction;
use simdash_state::{dash::Operation, routing::flooding::FloodingOp};

pub struct FloodingModel;

impl polestar::Machine for FloodingModel {
    type State = simdash_state::routing::flooding::FloodingState;
    type Action = simdash_action::routing::flooding::FloodingAction;
    type Fx = ();
    type Error = ();

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            FloodingAction::Tick => {
                state.ops.iter_mut().for_each(|op| op.tick());
                Ok((state, ()))
            }
            FloodingAction::StoreOp { op } => {
                state.ops.push(FloodingOp::new(&op));
                Ok((state, ()))
            }
            FloodingAction::TouchOp { op_id } => {
                state
                    .ops
                    .iter_mut()
                    .find(|op| op.id == op_id)
                    .map(|op| op.touch());
                Ok((state, ()))
            }
            FloodingAction::ReceiveOp { from, op } => {
                state.ops.push(FloodingOp::new(&op));
                Ok((state, ()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use simdash_state::routing::flooding::FloodingState;

    use super::*;

    #[test]
    fn test_flooding_model() {
        let model = FloodingModel;
        let state = FloodingState::default();
    }
}
