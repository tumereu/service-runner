use serde::{Deserialize, Serialize};
use crate::system_state::SystemState;

#[derive(Serialize, Deserialize)]
pub enum Action {
    Shutdown
}

#[derive(Serialize, Deserialize)]
pub enum Broadcast {
    State(SystemState)
}

pub trait Message {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &Vec<u8>) -> Self;
}

impl Message for Action {
    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn decode(bytes: &Vec<u8>) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}