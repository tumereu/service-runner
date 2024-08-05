use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::model::message::models::{Profile, ServiceStatus};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SystemState {
    pub status: Status,
    pub current_profile: Option<Profile>,
    pub service_statuses: HashMap<String, ServiceStatus>,
}
impl SystemState {
    pub fn new() -> SystemState {
        return SystemState {
            status: Status::Idle,
            current_profile: None,
            service_statuses: HashMap::new(),
        };
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Status {
    Idle,
    Exiting,
}
