use serde::{Deserialize, Serialize};

use crate::message::models::Profile;
use crate::system_state::SystemState;

#[derive(Serialize, Deserialize)]
pub enum Action {
    Shutdown,
    ActivateProfile(Profile),
}
impl AsRef<Action> for Action {
    fn as_ref(&self) -> &Action {
        self
    }
}

#[derive(Serialize, Deserialize)]
pub enum Broadcast {
    State(SystemState)
}
impl AsRef<Broadcast> for Broadcast {
    fn as_ref(&self) -> &Broadcast {
        self
    }
}

pub trait Message {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &Vec<u8>) -> Self;
}
impl<M> Message for M where M : Serialize + for<'de> Deserialize<'de> {
    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn decode(bytes: &Vec<u8>) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}