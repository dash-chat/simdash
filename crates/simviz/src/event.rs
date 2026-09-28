//! The wire protocol between the viz and a model's presenter.
//!
//! Frontend → server: [`Command`]. Server → frontend: [`Frame`], which carries
//! the topology and the [`VizEvent`]s the presenter produced for this step.

use serde::{Deserialize, Serialize};

use crate::{NodeId, Topology};

/// Something the frontend asks the viewer to do.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    /// Advance the simulation one step.
    StepForward,
    /// Go back one step.
    StepBack,
    /// Return to the initial state.
    Reset,
    /// A UI interaction to be interpreted by the presenter.
    Ui {
        /// The interaction.
        action: UiAction,
    },
}

/// A UI interaction. The viewer tracks node selection; everything else is
/// opaque to the viz and interpreted by the presenter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiAction {
    /// Select a node (or clear the selection with `None`).
    SelectNode {
        /// The node selected, or `None` to clear.
        node: Option<NodeId>,
    },
    /// A link emitted by the presenter was clicked. The payload is whatever
    /// the presenter put in the link.
    Link {
        /// Presenter-defined payload.
        payload: serde_json::Value,
    },
}

/// Visual style overrides for a node. `None` fields keep the default.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeStyle {
    /// Fill colour, any CSS colour string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Border colour.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    /// Opacity in `0.0..=1.0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    /// Label to draw instead of the node id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Visual style overrides for an edge. `None` fields keep the default.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeStyle {
    /// Line colour.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Line width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    /// Opacity in `0.0..=1.0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

/// A value shown in the sidebar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldValue {
    /// Plain text.
    Text {
        /// The text.
        text: String,
    },
    /// Clickable text that sends a [`UiAction`] back to the presenter.
    Link {
        /// The link text.
        text: String,
        /// Sent as `Command::Ui { action }` when clicked.
        action: UiAction,
    },
    /// A list of values, rendered inline.
    List {
        /// The items.
        items: Vec<FieldValue>,
    },
}

/// A labelled row in the sidebar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// Row label.
    pub label: String,
    /// Row value.
    pub value: FieldValue,
}

/// An instruction from the presenter to the viz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VizEvent {
    /// Style one node.
    NodeStyle {
        /// The node.
        node: NodeId,
        /// The style.
        style: NodeStyle,
    },
    /// Style one directed edge.
    EdgeStyle {
        /// Edge source.
        source: NodeId,
        /// Edge target.
        target: NodeId,
        /// The style.
        style: EdgeStyle,
    },
    /// Replace the sidebar contents.
    Sidebar {
        /// Optional heading; the frontend falls back to the selected node.
        title: Option<String>,
        /// Rows to show.
        fields: Vec<Field>,
    },
}

/// Everything the frontend needs to draw one simulation step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// Steps taken from the initial state.
    pub step: usize,
    /// Graph shape at this step.
    pub topology: Topology,
    /// Node the viewer currently has selected.
    pub selected: Option<NodeId>,
    /// Decorations produced by the presenter for this step.
    pub events: Vec<VizEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_json_shape() {
        let cmd = Command::Ui {
            action: UiAction::SelectNode {
                node: Some(NodeId(3)),
            },
        };
        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "Ui", "action": { "type": "SelectNode", "node": 3 } })
        );
        let back: Command = serde_json::from_value(json).unwrap();
        assert_eq!(back, cmd);
    }

    #[test]
    fn plain_commands_round_trip() {
        for cmd in [Command::StepForward, Command::StepBack, Command::Reset] {
            let json = serde_json::to_value(&cmd).unwrap();
            assert_eq!(serde_json::from_value::<Command>(json).unwrap(), cmd);
        }
    }

    #[test]
    fn link_payload_is_opaque_json() {
        let cmd = Command::Ui {
            action: UiAction::Link {
                payload: serde_json::json!({ "op": 7 }),
            },
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"op\":7"));
        assert_eq!(serde_json::from_str::<Command>(&json).unwrap(), cmd);
    }

    #[test]
    fn node_style_omits_unset_fields() {
        let ev = VizEvent::NodeStyle {
            node: NodeId(1),
            style: NodeStyle {
                color: Some("#f00".into()),
                ..Default::default()
            },
        };
        assert_eq!(
            serde_json::to_value(&ev).unwrap(),
            serde_json::json!({ "type": "NodeStyle", "node": 1, "style": { "color": "#f00" } })
        );
    }
}
