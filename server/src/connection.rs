use std::collections::VecDeque;
use std::fmt::format;
use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use shared::dbg_println;

use shared::message::{Action, Broadcast, MessageTransmitter};
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn run_server(port: u16, server: Arc<Mutex<ServerState>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut client_count: u32 = 0;

    while server.lock().unwrap().get_state().status != Status::Exiting {
        let stream = listener.accept();
        client_count += 1;

        match stream {
            Ok((stream, _)) => {
                handle_connection(stream, client_count, server.clone());
                {
                    let mut state = server.lock().unwrap();
                    // Send the current system state to the connected client
                    let broadcast = Broadcast::State(state.get_state().clone());
                    state.broadcast_one(client_count, broadcast);
                    // Send all so-far accumulated outputs to the client
                    let broadcast = Broadcast::OutputSync(state.output_store.clone());
                    state.broadcast_one(client_count, broadcast);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("Encountered an unexpected IO error {e}"),
        }
    }
}

pub fn handle_connection(mut stream: TcpStream, index: u32, server: Arc<Mutex<ServerState>>) {
    server
        .lock()
        .unwrap()
        .broadcasts_out
        .insert(index, VecDeque::new());

    let handle = {
        let server = server.clone();
        thread::spawn(move || {
            while server.lock().unwrap().get_state().status != Status::Exiting {
                while stream.has_incoming(Duration::from_millis(10)).unwrap() {
                    let incoming: Action = stream.receive().unwrap();
                    server.lock().unwrap().actions_in.push_back(incoming);
                }
                while let Some(outgoing) = server
                    .lock()
                    .unwrap()
                    .broadcasts_out
                    .get_mut(&index)
                    .unwrap()
                    .pop_front()
                {
                    if let Err(error) = stream.send(outgoing) {
                        dbg_println!("Error occurred when sending to a stream {error:?}")
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }

            server.lock().unwrap().broadcasts_out.remove(&index);

            stream.shutdown(Shutdown::Both).unwrap();
        })
    };

    server.lock().unwrap().active_threads.push((format!("connection-handler-{index}"), handle));
}
