extern crate core;

use std::{env, thread};
use std::error::Error;
use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shared::config::{Config, read_config};
use shared::message::{Action, Message, MessageTransmitter};
use shared::system_state::{Status, SystemState};

use crate::action_processor::start_action_processor;
use crate::connection::run_server;
use crate::server_state::ServerState;

mod server_state;
mod connection;
mod action_processor;

fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::args().collect::<Vec<String>>()
        .get(1)
        .unwrap()
        .parse()?;

    let state = Arc::new(Mutex::new(ServerState::new()));

    start_action_processor(state.clone());
    run_server(port, state.clone());

    Ok(())
}