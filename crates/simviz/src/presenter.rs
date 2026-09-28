//! The seam: how one model is shown, without the viz knowing the model.

use crate::{Topology, UiAction, VizEvent};

/// Per-model UI module.
///
/// A presenter turns model state into the things the viz can draw. Only
/// [`topology`](Presenter::topology) is required; the defaults make a
/// presenter that shows a bare graph with no colours and an empty sidebar.
///
/// The presenter owns any UI-side state it needs (which op is being traced,
/// which group is highlighted). [`on_action`](Presenter::on_action) updates
/// that state; [`render`](Presenter::render) produces the full decoration set
/// for the current model state and is called after every command.
pub trait Presenter: Send + Sync + 'static {
    /// The model state this presenter reads.
    type State;

    /// The graph shape for this state.
    fn topology(&self, state: &Self::State) -> Topology;

    /// React to a UI interaction. Default: ignore it.
    fn on_action(&mut self, state: &Self::State, action: &UiAction) {
        let _ = (state, action);
    }

    /// Produce the decorations for this state. Default: none.
    fn render(&self, state: &Self::State) -> Vec<VizEvent> {
        let _ = state;
        vec![]
    }
}

/// A presenter built from a topology closure, with noop action handling and
/// rendering. Handy for tests and minimal adapters.
pub struct FnPresenter<S, F> {
    topology: F,
    _state: std::marker::PhantomData<fn(&S)>,
}

impl<S, F> FnPresenter<S, F>
where
    F: Fn(&S) -> Topology,
{
    /// Wrap a closure that extracts the topology from a state.
    pub fn new(topology: F) -> Self {
        Self {
            topology,
            _state: std::marker::PhantomData,
        }
    }
}

impl<S, F> Presenter for FnPresenter<S, F>
where
    S: 'static,
    F: Fn(&S) -> Topology + Send + Sync + 'static,
{
    type State = S;

    fn topology(&self, state: &S) -> Topology {
        (self.topology)(state)
    }
}
