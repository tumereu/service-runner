//! ui crate: placeholder library

// This is a plain library file for the `ui` crate.
// Add public API here when needed.

pub mod component;
pub mod space;

mod frame_ctx;
mod renderer;
mod signal;
mod state_store;

pub use frame_ctx::*;
pub use renderer::*;
pub use signal::*;
pub use state_store::*;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum UIError {
    Nested {
        component_key: String,
        component_type: String,
        error: Box<UIError>,
    },
    InvalidProp {
        msg: String,
    },
    IllegalState {
        msg: String,
    },
    InvalidRenderArgs {
        msg: String,
    },
    User {
        error: Box<dyn Error>,
    },
    IO(std::io::Error),
}
impl UIError {
    pub fn unwrap_nested(&self) -> (Vec<(&str, &str)>, &UIError) {
        let mut path = Vec::new();
        let mut current = self;

        loop {
            match current {
                UIError::Nested {
                    component_key,
                    component_type,
                    error,
                } => {
                    path.push((component_key.as_str(), component_type.as_str()));
                    current = error.as_ref();
                }
                _ => break,
            }
        }

        (path, current)
    }
}

impl Display for UIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UIError::InvalidProp { msg } => write!(f, "Invalid prop: {}", msg),
            UIError::IllegalState { msg } => write!(f, "Illegal state: {}", msg),
            UIError::InvalidRenderArgs { msg } => write!(f, "Invalid render arg: {}", msg),
            UIError::IO(e) => write!(f, "IO error: {}", e),
            UIError::User { error } => error.fmt(f),
            nested @ UIError::Nested { .. } => {
                let (path, tail) = nested.unwrap_nested();
                write!(f, "Nested error: ")?;
                for (key, component_type) in path.iter() {
                    write!(f, "{}[key={}] -> ", component_type, key)?;
                }
                tail.fmt(f)
            }
        }
    }
}

impl Error for UIError {}

pub type UIResult<T> = Result<T, UIError>;
