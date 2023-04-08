use std::{env, thread};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::client_state::ClientState;
use crate::connection::handle_stream;

pub fn connect_to_server(state: Arc<Mutex<ClientState>>) -> Result<JoinHandle<std::io::Result<()>>, String> {
    let port = state.lock().unwrap().config.server.port;

    fn open_stream(port: u16) -> Option<TcpStream> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let result = TcpStream::connect_timeout(&addr, Duration::from_millis(1000));

        if let Ok(stream) = result {
            Some(stream)
        } else {
            None
        }
    }

    let stream = open_stream(port);
    let stream = if stream.is_none() {
        Command::new(&state.lock().unwrap().config.server.executable)
            .arg(&state.lock().unwrap().config.conf_dir)
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

        // TODO implement better
        thread::sleep(Duration::from_millis(1000));

        open_stream(port)
    } else {
        stream
    };

    Ok(
        handle_stream(
            stream.ok_or(format!("Could not connect to server on port {port}"))?,
            state.clone()
        )
    )
}
