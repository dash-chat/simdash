pub mod chat;
pub mod network;
pub mod node;
pub mod routing;

use polestar::Machine;

pub trait Behavior<M>
where
    M: Machine<Error = anyhow::Error>,
{
    fn next(&mut self, state: &M::State) -> anyhow::Result<Option<M::Action>>;
}
