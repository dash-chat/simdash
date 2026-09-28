//! A simulation paired with a presenter, driven by commands and producing frames.

use polestar::{Behavior, FxOf, StateOf};

use crate::{Command, Frame, NodeId, Presenter, Simulation, UiAction};

/// A [`Simulation`] plus the [`Presenter`] that knows how to show it.
///
/// The viewer tracks which node is selected so that a presenter with the
/// default noop `on_action` still yields a usable frame.
pub struct Viewer<B, P>
where
    B: Behavior,
    P: Presenter<State = StateOf<B::Model>>,
{
    sim: Simulation<B>,
    presenter: P,
    selected: Option<NodeId>,
}

impl<B, P> Viewer<B, P>
where
    B: Behavior,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    /// Pair a simulation with its presenter.
    pub fn new(sim: Simulation<B>, presenter: P) -> Self {
        Self {
            sim,
            presenter,
            selected: None,
        }
    }

    /// The underlying simulation.
    pub fn simulation(&self) -> &Simulation<B> {
        &self.sim
    }

    /// The presenter.
    pub fn presenter(&self) -> &P {
        &self.presenter
    }

    /// Render the current step.
    pub fn frame(&self) -> Frame {
        let state = self.sim.state();
        Frame {
            step: self.sim.step(),
            topology: self.presenter.topology(state),
            selected: self.selected,
            events: self.presenter.render(state),
        }
    }

    /// Apply a command and render the result.
    pub fn handle(&mut self, command: Command) -> anyhow::Result<Frame> {
        match command {
            Command::StepForward => self.sim.step_forward()?,
            Command::StepBack => self.sim.step_back()?,
            Command::Reset => {
                self.sim.reset();
                self.selected = None;
            }
            Command::Ui { action } => {
                if let UiAction::SelectNode { node } = &action {
                    self.selected = *node;
                }
                self.presenter.on_action(self.sim.state(), &action);
            }
        }
        Ok(self.frame())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::toy::{Ring, RingBehavior, ring_simulation};
    use crate::{Field, FieldValue, FnPresenter, NodeStyle, Topology, VizEvent};
    use std::sync::{Arc, Mutex};

    fn viewer(n: u32) -> Viewer<RingBehavior, FnPresenter<Ring, fn(&Ring) -> Topology>> {
        Viewer::new(ring_simulation(n), FnPresenter::new(Ring::topology))
    }

    #[test]
    fn initial_frame_has_topology_and_no_events() {
        let v = viewer(3);
        let f = v.frame();
        assert_eq!(f.step, 0);
        assert_eq!(f.topology.nodes().len(), 3);
        assert!(f.topology.edges().is_empty());
        assert_eq!(f.selected, None);
        assert!(f.events.is_empty());
    }

    #[test]
    fn playback_commands_move_the_simulation() {
        let mut v = viewer(3);
        let f = v.handle(Command::StepForward).unwrap();
        assert_eq!(f.step, 1);
        assert_eq!(f.topology.edges().len(), 1);
        let f = v.handle(Command::StepForward).unwrap();
        assert_eq!(f.step, 2);
        let f = v.handle(Command::StepBack).unwrap();
        assert_eq!(f.step, 1);
        let f = v.handle(Command::Reset).unwrap();
        assert_eq!(f.step, 0);
    }

    #[test]
    fn selection_is_tracked_and_cleared_on_reset() {
        let mut v = viewer(3);
        let f = v
            .handle(Command::Ui {
                action: UiAction::SelectNode {
                    node: Some(NodeId(1)),
                },
            })
            .unwrap();
        assert_eq!(f.selected, Some(NodeId(1)));
        let f = v.handle(Command::StepForward).unwrap();
        assert_eq!(f.selected, Some(NodeId(1)), "selection survives stepping");
        let f = v.handle(Command::Reset).unwrap();
        assert_eq!(f.selected, None);
    }

    /// A presenter that colours the newest edge's source and shows a sidebar
    /// with a link that, when clicked, records the payload.
    struct Recording {
        clicked: Arc<Mutex<Vec<serde_json::Value>>>,
    }

    impl Presenter for Recording {
        type State = Ring;

        fn topology(&self, state: &Ring) -> Topology {
            state.topology()
        }

        fn on_action(&mut self, _: &Ring, action: &UiAction) {
            if let UiAction::Link { payload } = action {
                self.clicked.lock().unwrap().push(payload.clone());
            }
        }

        fn render(&self, state: &Ring) -> Vec<VizEvent> {
            let mut events = vec![];
            if let Some(&(a, _)) = state.edges.last() {
                events.push(VizEvent::NodeStyle {
                    node: NodeId(a),
                    style: NodeStyle {
                        color: Some("#0f0".into()),
                        ..Default::default()
                    },
                });
            }
            events.push(VizEvent::Sidebar {
                title: None,
                fields: vec![Field {
                    label: "edges".into(),
                    value: FieldValue::Link {
                        text: state.edges.len().to_string(),
                        action: UiAction::Link {
                            payload: serde_json::json!({ "edges": state.edges.len() }),
                        },
                    },
                }],
            });
            events
        }
    }

    #[test]
    fn presenter_events_flow_into_frames_and_links_flow_back() {
        let clicked = Arc::new(Mutex::new(vec![]));
        let mut v = Viewer::new(
            ring_simulation(3),
            Recording {
                clicked: clicked.clone(),
            },
        );
        let f = v.handle(Command::StepForward).unwrap();
        assert!(matches!(
            f.events[0],
            VizEvent::NodeStyle {
                node: NodeId(0),
                ..
            }
        ));
        let VizEvent::Sidebar { fields, .. } = &f.events[1] else {
            panic!("expected sidebar");
        };
        let FieldValue::Link { action, .. } = &fields[0].value else {
            panic!("expected link");
        };
        v.handle(Command::Ui {
            action: action.clone(),
        })
        .unwrap();
        assert_eq!(
            *clicked.lock().unwrap(),
            vec![serde_json::json!({ "edges": 1 })]
        );
    }
}
