#![allow(unused)]

mod dash;
mod node;
mod routing;

pub use dash::*;
pub use node::*;
pub use routing::*;

pub struct Model {
    node: NodeModel,
}

impl polestar::Machine for Model {
    type State = simdash_state::State;
    type Action = simdash_action::Action;
    type Fx = ();
    type Error = ();

    fn transition(
        &self,
        state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        todo!()
    }
}
