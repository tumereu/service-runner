use std::ptr::copy_nonoverlapping;
use serde::{Deserialize, Serialize};
use crate::config_parsing::{Profile, Service};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SystemState {
    pub status: Status,
    pub current_profile: Option<(Profile, Vec<Service>)>,

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