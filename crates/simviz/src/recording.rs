//! Static recordings: a sequence of frames the frontend can replay without a server.

use polestar::{Behavior, FxOf, StateOf};
use serde::{Deserialize, Serialize};

use crate::{Frame, Presenter, Viewer};

/// A replayable sequence of frames, one per step starting at step 0.
///
/// Recorded with no UI interaction, so `selected` is `None` throughout and
/// presenter events reflect only the model state. Serve the JSON next to the
/// frontend and open it with `?recording=<url>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    /// Frames for steps `0..=steps`.
    pub frames: Vec<Frame>,
}

impl<B, P> Viewer<B, P>
where
    B: Behavior,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    /// Reset, capture a frame at every step up to and including `steps`, then
    /// reset again so the viewer is left at its initial state.
    pub fn record(&mut self, steps: usize) -> anyhow::Result<Recording> {
        self.handle(crate::Command::Reset)?;
        let mut frames = Vec::with_capacity(steps + 1);
        frames.push(self.frame());
        for _ in 0..steps {
            frames.push(self.handle(crate::Command::StepForward)?);
        }
        self.handle(crate::Command::Reset)?;
        Ok(Recording { frames })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::toy::{Ring, ring_simulation};
    use crate::{Command, FnPresenter, NodeId, Topology, UiAction};

    fn viewer() -> Viewer<crate::simulation::toy::RingBehavior, FnPresenter<Ring, fn(&Ring) -> Topology>> {
        Viewer::new(ring_simulation(4), FnPresenter::new(Ring::topology))
    }

    #[test]
    fn records_one_frame_per_step_from_zero() {
        let rec = viewer().record(3).unwrap();
        assert_eq!(rec.frames.len(), 4);
        assert_eq!(
            rec.frames.iter().map(|f| f.step).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            rec.frames.iter().map(|f| f.topology.edges().len()).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
    }

    #[test]
    fn recording_starts_from_initial_state_and_leaves_viewer_reset() {
        let mut v = viewer();
        v.handle(Command::StepForward).unwrap();
        v.handle(Command::Ui {
            action: UiAction::SelectNode {
                node: Some(NodeId(2)),
            },
        })
        .unwrap();
        let rec = v.record(2).unwrap();
        assert_eq!(rec.frames[0].step, 0);
        assert!(rec.frames.iter().all(|f| f.selected.is_none()));
        assert_eq!(v.frame().step, 0);
    }

    #[test]
    fn recording_round_trips_through_json() {
        let rec = viewer().record(2).unwrap();
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(serde_json::from_str::<Recording>(&json).unwrap(), rec);
    }

    #[test]
    fn model_error_during_recording_surfaces() {
        assert!(viewer().record(10).is_err(), "ring of 4 completes at step 4");
    }
}
