use serde::{Deserialize, Serialize};

use crate::message::models::Profile;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SystemState {
    pub status: Status,
    pub current_profile: Option<Profile>,

}
impl SystemState {
    pub fn new() -> SystemState {
        return SystemState {
            status: Status::Idle,
            current_profile: None
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Status {
    Idle,
    Exiting
}