use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::message::{Action, Broadcast, MessageTransmitter};
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn run_server(port: u16, state: Arc<Mutex<ServerState>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut client_count: u32 = 0;

    while state.lock().unwrap().system_state.status != Status::Exiting {
        let stream = listener.accept();
        client_count += 1;

        match stream {
            Ok((stream, _)) => {
                handle_connection(stream, client_count, state.clone());
                // Whenever a client connects, send the updated system state to all clients
                {
                    let mut state = state.lock().unwrap();
                    let broadcast = Broadcast::State(state.system_state.clone());
                    state.broadcast_one(client_count, broadcast);
                }
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("Encountered an unexpected IO error {e}")
        }
    }
}

pub fn handle_connection(
    mut stream: TcpStream,
    index: u32,
    state: Arc<Mutex<ServerState>>
) {
    state.lock().unwrap().broadcasts_out.insert(index, Vec::new());

    thread::spawn(move || {
        while state.lock().unwrap().system_state.status != Status::Exiting {
            while stream.has_incoming(Duration::from_millis(10))? {
                let incoming: Action = stream.receive()?;
                state.lock().unwrap().actions_in.push(incoming);
            }
            while let Some(outgoing) = state.lock().unwrap().broadcasts_out.get_mut(&index).unwrap().pop() {
                println!("Sending a broadcast");
                stream.send(outgoing)?;
            }

            thread::sleep(Duration::from_millis(1));
        }

        state.lock().unwrap().broadcasts_out.remove(&index);

        stream.shutdown(Shutdown::Both)?;

        Ok::<(), std::io::Error>(())
    });
}

