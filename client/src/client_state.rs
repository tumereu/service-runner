use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use shared::config::Config;
use shared::message::{Action, Broadcast};
use shared::system_state::SystemState;
use crate::ui::UIState;

pub struct ClientState {
    pub status: Status,
    pub system: Option<SystemState>,
    pub actions_out: Vec<Action>,
    pub broadcasts_in: Vec<Broadcast>,
    pub ui: UIState,
    pub config: Config,
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        ClientState {
            status: Status::Ready,
            system: None,
            actions_out: Vec::new(),
            broadcasts_in: Vec::new(),
            ui: UIState::Initializing,
            config,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Status {
    Ready,
    Exiting,
}