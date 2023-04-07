use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Instant};
use shared::config::Config;

use shared::system_state::SystemState;

pub struct ClientState {
    pub status: Arc<Mutex<Status>>,
    pub system: Arc<Mutex<Option<SystemState>>>,
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub config: Arc<Config>,
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        ClientState {
            status: Arc::new(Mutex::new(Status::Ready)),
            system: Arc::new(Mutex::new(None)),
            stream: Arc::new(Mutex::new(None)),
            config: Arc::new(config),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Status {
    Ready,
    Exiting,
}