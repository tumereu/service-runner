use std::error::Error;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::system_state::SystemState;

#[derive(Serialize, Deserialize)]
pub enum Action {
    Shutdown
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

pub trait MessageTransmitter<E : Error> {
    fn send<M, R>(&mut self, msg: R) -> Result<(), E> where M : Message, R: AsRef<M>;
    fn receive<M>(&mut self) -> Result<M, E> where M : Message;
    fn has_incoming(&self, block_for: Duration) -> Result<bool, E>;
}

impl MessageTransmitter<std::io::Error> for TcpStream {
    fn send<M, R>(&mut self, msg: R) -> Result<(), std::io::Error> where M: Message, R: AsRef<M> {
        let bytes = msg.as_ref().encode();

        let len = bytes.len() as u64;

        self.write(&len.to_be_bytes())?;
        self.write(&bytes)?;

        Ok(())
    }

    fn receive<M>(&mut self) -> Result<M, std::io::Error> where M: Message {
        self.set_read_timeout(None)?;

        let mut len_bytes = [0 as u8; size_of::<u64>()];
        self.read_exact(&mut len_bytes)?;

        let len: usize = u64::from_be_bytes(len_bytes).try_into().unwrap();

        let mut msg_bytes: Vec<u8> = vec![0; len];
        self.read(&mut msg_bytes)?;

        Ok(M::decode(&msg_bytes))
    }

    fn has_incoming(&self, block_for: Duration) -> Result<bool, std::io::Error> {
        let mut len_bytes = [0 as u8; size_of::<u64>()];
        self.set_read_timeout(Some(block_for))?;

        if let Ok(num_read) = self.peek(&mut len_bytes) {
            Ok(num_read > 0)
        } else {
            Ok(false)
        }
    }
}