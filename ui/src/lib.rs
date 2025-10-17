//! ui crate: placeholder library

// This is a plain library file for the `ui` crate.
// Add public API here when needed.

pub mod component;
pub mod space;
pub mod input;

mod frame_ctx;
mod renderer;
mod signals;
mod state_store;

pub use frame_ctx::*;
pub use renderer::*;
pub use signals::*;
pub use state_store::*;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum UIError {
    Nested {
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
    MissingAttr {
        attr: String,
    },
    User {
        error: Box<dyn Error>,
    },
    IO(std::io::Error),
}
impl UIError {
    pub fn unwrap_nested(&self) -> (Vec<&str>, &UIError) {
        let mut path = Vec::new();
        let mut current = self;

        loop {
            match current {
                UIError::Nested {
                    component_type,
                    error,
                } => {
                    path.push(component_type.as_str());
                    current = error.as_ref();
                }
                _ => break,
            }
        }

        (path, current)
    }

    pub fn nested<T>(self) -> Self {
        let full = std::any::type_name::<T>();
        // Remove generics
        let cleaned = match full.find('<') {
            Some(pos) => &full[..pos],
            None => full,
        };

        UIError::Nested {
            component_type: cleaned.to_string(),
            error: Box::new(self),
        }
    }
}

impl Display for UIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UIError::InvalidProp { msg } => write!(f, "Invalid prop: {}", msg),
            UIError::IllegalState { msg } => write!(f, "Illegal state: {}", msg),
            UIError::InvalidRenderArgs { msg } => write!(f, "Invalid render arg: {}", msg),
            UIError::MissingAttr { attr } => write!(f, "Renderer does not have the required attribute '{}' set", attr),
            UIError::IO(e) => write!(f, "IO error: {}", e),
            UIError::User { error } => error.fmt(f),
            nested @ UIError::Nested { .. } => {
                let (path, tail) = nested.unwrap_nested();
                write!(f, "Nested error: ")?;
                for component_type in path.iter() {
                    write!(f, "{} -> ", component_type)?;
                }
                tail.fmt(f)
            }
        }
    }
}

impl Error for UIError {}

pub type UIResult<T> = Result<T, UIError>;
