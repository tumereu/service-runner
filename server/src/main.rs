extern crate core;

use std::{env, thread};
use std::error::Error;
use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shared::config::{Config, read_config};
use shared::system_state::{Status, SystemState};

fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args().collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();

    let config = Arc::new(read_config(&config_dir)?);
    let port = config.server.port;
    let state = Arc::new(Mutex::new(SystemState::new()));

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    while state.lock().unwrap().status != Status::Exiting {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_connection(stream, config.clone(), state.clone()),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("Encountered an unexpected IO error {e}")
            }
        }
    }

    Ok(())
}

fn handle_connection(
    stream: TcpStream,
    _config: Arc<Config>,
    state: Arc<Mutex<SystemState>>
) {
    thread::spawn(move || {
        while state.lock().unwrap().status != Status::Exiting {

        }

        stream.shutdown(Shutdown::Both)?;

        Ok::<(), std::io::Error>(())
    });
}