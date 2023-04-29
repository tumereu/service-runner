use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, thread};

use crate::client_state::ClientState;
use crate::connection::handle_stream;

pub fn connect_to_server(
    state: Arc<Mutex<ClientState>>,
) -> Result<JoinHandle<std::io::Result<()>>, String> {
    let stream = {
        let state = state.lock().unwrap();
        let port = state.config.server.port;

        fn open_stream(port: u16) -> Option<TcpStream> {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let result = TcpStream::connect_timeout(&addr, Duration::from_millis(1000));

            if let Ok(stream) = result {
                Some(stream)
            } else {
                None
            }
        }

        let mut stream = open_stream(port);

        if stream.is_none() {
            Command::new(&state.config.server.executable)
                .arg(&state.config.server.port.to_string())
                .current_dir(env::current_dir().map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to read current workdir: {msg}")
                })?)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to spawn server process: {msg}")
                })?;
        }

        while stream.is_none() {
            thread::sleep(Duration::from_millis(10));
            stream = open_stream(port);
        }

        stream.ok_or(format!("Could not connect to server on port {port}"))?
    };

    Ok(handle_stream(stream, state.clone()))
}
