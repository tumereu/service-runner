

use std::time::{Instant};


use shared::system_state::SystemState;

pub struct ClientState {
    pub status: Status,
    pub system: Option<SystemState>,
    pub last_status_check: Instant
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState {
            status: Status::CheckingServerStatus,
            system: None,
            last_status_check: Instant::now()
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Status {
    CheckingServerStatus,
    StartingServer,
    Ready,
    Exiting,
}