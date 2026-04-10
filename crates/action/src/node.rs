use crate::chat::ChatAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeAction<RA> {
    Chat(ChatAction),
    Routing(RA),
    Tick,
}
