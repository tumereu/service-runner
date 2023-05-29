use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use shared::dbg_println;

use shared::message::{Action, Broadcast, MessageTransmitter};

use crate::client_state::{ClientState, ClientStatus};

pub fn handle_stream(
    mut stream: TcpStream,
    state: Arc<Mutex<ClientState>>,
) -> thread::JoinHandle<std::io::Result<()>> {
    thread::spawn(move || {
        let mut stream_ok = true;

        while state.lock().unwrap().status != ClientStatus::Exiting && stream_ok {
            while stream.has_incoming(Duration::from_millis(10))? {
                match stream.receive() {
                    Ok(incoming) => {
                        state.lock().unwrap().broadcasts_in.push_back(incoming);
                    }
                    Err(error) => {
                        dbg_println!("Error in receiving a broadcast from the server: {error:?}");
                        stream_ok = false;
                        break;
                    }
                }
            }
            while let Some(outgoing) = state.lock().unwrap().actions_out.pop_front() {
                if stream_ok {
                    stream.send(outgoing)?;
                }
            }

            if stream_ok {
                // ticket/send data through the stream. If the connection is broken, then this will fail and the thread will
                // exit.
                stream.send(Action::Tick)?;
            }
        }

        if !state.lock().unwrap().config.server.daemon && stream_ok {
            stream.send(Action::Shutdown)?;
        }

        stream.shutdown(Shutdown::Both)?;

        Ok::<(), std::io::Error>(())
    })
}
