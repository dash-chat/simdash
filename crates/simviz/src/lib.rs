//! simviz: a generic seam for visualizing replayable simulations of a
//! changing graph of nodes.
//!
//! The viz knows only that there is a [`Simulation`] which can be stepped
//! forward, back and reset, and that at any step the model state can be
//! rendered as a [`Topology`]. Everything model-specific goes through a
//! [`Presenter`], which turns model state into [`VizEvent`]s (node colours,
//! edge styles, sidebar fields) and interprets [`UiAction`]s coming back
//! from the frontend.

mod event;
mod presenter;
mod server;
mod simulation;
mod topology;
mod viewer;

pub use event::{Command, EdgeStyle, Field, FieldValue, Frame, NodeStyle, UiAction, VizEvent};
pub use presenter::{FnPresenter, Presenter};
pub use server::{router, serve};
pub use simulation::Simulation;
pub use topology::{Edge, NodeId, Topology};
pub use viewer::Viewer;
