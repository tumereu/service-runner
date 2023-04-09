use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


use shared::message::{Action, Broadcast, MessageTransmitter};


use crate::client_state::{ClientState, ClientStatus};

pub fn handle_stream(
    mut stream: TcpStream,
    state: Arc<Mutex<ClientState>>
) -> thread::JoinHandle<std::io::Result<()>> {
    thread::spawn(move || {
        while state.lock().unwrap().status != ClientStatus::Exiting {
            while stream.has_incoming(Duration::from_millis(10))? {
                let incoming: Broadcast = stream.receive()?;
                state.lock().unwrap().broadcasts_in.push(incoming);
            }
            while let Some(outgoing) = state.lock().unwrap().actions_out.pop() {
                stream.send(outgoing)?;
            }
        }

        if !state.lock().unwrap().config.server.daemon {
            stream.send(Action::Shutdown)?;
        }

        stream.shutdown(Shutdown::Both)?;

        Ok::<(), std::io::Error>(())
    })
}