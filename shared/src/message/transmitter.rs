use std::error::Error;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::time::Duration;
use bincode::ErrorKind;
use crate::dbg_println;

use crate::message::Message;

pub trait MessageTransmitter<E: Error> {
    fn send<M, R>(&mut self, msg: R) -> Result<(), E>
    where
        M: Message,
        R: AsRef<M>;
    fn receive<M>(&mut self) -> Result<M, E>
    where
        M: Message;
    fn has_incoming(&self, block_for: Duration) -> Result<bool, E>;
}

impl MessageTransmitter<std::io::Error> for TcpStream {
    fn send<M, R>(&mut self, msg: R) -> Result<(), std::io::Error>
    where
        M: Message,
        R: AsRef<M>,
    {
        let bytes = msg.as_ref().encode();

        let len = bytes.len() as u64;

        self.write(&len.to_be_bytes())?;
        self.write(&bytes)?;

        Ok(())
    }

    fn receive<M>(&mut self) -> Result<M, std::io::Error>
    where
        M: Message,
    {
        self.set_read_timeout(None)?;

        let mut len_bytes = [0 as u8; size_of::<u64>()];
        self.read_exact(&mut len_bytes)?;

        let len: usize = u64::from_be_bytes(len_bytes).try_into().unwrap();

        if len > 1024 * 1024 {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Unreasonable message size: {len} bytes ({len_bytes:?})")
                )
            )
        } else {
            let mut msg_bytes: Vec<u8> = vec![0; len];
            self.read_exact(&mut msg_bytes)?;

            Ok(M::decode(&msg_bytes))
        }
    }

    fn has_incoming(&self, block_for: Duration) -> Result<bool, std::io::Error> {
        let mut len_bytes = [0 as u8; size_of::<u64>()];
        self.set_read_timeout(Some(block_for))?;

        if let Ok(num_read) = self.peek(&mut len_bytes) {
            Ok(num_read == 8)
        } else {
            Ok(false)
        }
    }
}
