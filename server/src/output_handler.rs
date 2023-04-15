use std::sync::{Arc, Mutex};
use std::time::Instant;
use shared::message::models::{OutputKey, OutputLine};
use shared::system_state::SystemState;
use crate::server_state::ServerState;
use once_cell::sync::Lazy;

static REFERENCE_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());

pub fn process_output_line<K>(
    state: Arc<Mutex<ServerState>>,
    key: K,
    output: String
) where K : AsRef<OutputKey> {
    let mut state = state.lock().unwrap();

    let output_line = OutputLine {
        value: output,
        timestamp: Instant::now().duration_since(*REFERENCE_INSTANT).as_millis()
    };

    state.output_store.add_output(key.as_ref(), output_line);
}