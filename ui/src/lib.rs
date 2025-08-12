//! ui crate: placeholder library

// This is a plain library file for the `ui` crate.
// Add public API here when needed.

pub mod component;
pub mod space;

mod renderer;
mod frame_ctx;
mod state_store;
mod signal;

pub use renderer::*;
pub use frame_ctx::*;
pub use state_store::*;
pub use signal::*;

pub enum RenderError {
    ComponentArg { msg: String },
    ComponentState { msg: String },
    RenderArg { msg: String },
}

pub type UIResult<T> = Result<T, RenderError>;