//! simviz: a generic seam for visualizing replayable simulations of a
//! changing graph of nodes.
//!
//! The viz knows only that there is a [`Simulation`] which can be stepped
//! forward, back and reset, and that at any step the model state can be
//! rendered as a [`Topology`]. Everything model-specific goes through a
//! [`Presenter`], which turns model state into [`VizEvent`]s (node colours,
//! edge styles, moons around nodes, sidebar fields) and interprets [`UiAction`]s coming back
//! from the frontend.
//!
//! [`serve`] runs the interactive viewer with the embedded frontend.
//! [`Viewer::record`] produces a [`Recording`] the frontend can replay from a
//! static file instead, with [`export_ui`] writing that frontend out to host it.

mod event;
mod presenter;
mod recording;
mod server;
mod simulation;
mod topology;
mod viewer;

pub use event::{
    Command, EdgeStyle, Field, FieldValue, Frame, MOON_RADIUS_MAX, MOON_RADIUS_MIN, MOON_SLOTS,
    MoonStyle, NodeStyle, UiAction, VizEvent,
};
pub use presenter::{FnPresenter, Presenter};
pub use recording::Recording;
pub use server::{export_ui, router, serve};
pub use simulation::Simulation;
pub use topology::{Edge, NodeId, Topology};
pub use viewer::Viewer;
