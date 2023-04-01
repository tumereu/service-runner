use std::collections::HashMap;
use std::future::Future;
use reqwest::Response;
use shared::config::Config;

pub struct ClientState {
    pub status: Status,
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState {
            status: Status::CheckingServerStatus,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Status {
    CheckingServerStatus,
    StartingServer,
    Idle,
    Finishing,
    ReadyToExit,
}