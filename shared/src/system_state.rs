use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SystemState {
    pub status: Status
}
impl SystemState {
    pub fn new() -> SystemState {
        return SystemState {
            status: Status::Idle
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Status {
    Idle,
    Exiting
}