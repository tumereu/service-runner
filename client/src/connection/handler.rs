use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::config::Config;
use shared::message::{Action, Broadcast, MessageTransmitter};
use shared::system_state::SystemState;

use crate::client_state::{ClientState, Status};

pub fn handle_stream(
    mut stream: TcpStream,
    state: Arc<Mutex<ClientState>>
) -> thread::JoinHandle<std::io::Result<()>> {
    thread::spawn(move || {
        while state.lock().unwrap().status != Status::Exiting {
            while stream.has_incoming()? {
                let incoming: Broadcast = stream.receive()?;
                state.lock().unwrap().broadcasts_in.push(incoming);
            }
            while let Some(outgoing) = state.lock().unwrap().actions_out.pop() {
                stream.send(outgoing)?;
            }

            thread::sleep(Duration::from_millis(1))
        }

        if !state.lock().unwrap().config.server.daemon {
            stream.send(Action::Shutdown)?;
        }

        stream.shutdown(Shutdown::Both)?;

        Ok::<(), std::io::Error>(())
    })
}