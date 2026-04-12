use polestar::Machine;

pub trait Behavior<M>
where
    M: Machine<Error = anyhow::Error>,
{
    fn next(&mut self, state: &M::State) -> anyhow::Result<Vec<M::Action>>;
}
