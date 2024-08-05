extern crate core;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use itertools::Itertools;

use shared::dbg_println;
use shared::system_state::Status;

use crate::action_processor::start_action_processor;
use crate::file_watcher::start_file_watcher;
use crate::server_state::ServerState;
use crate::service_worker::start_service_worker;

pub mod action_processor;
pub mod server_state;
pub mod service_worker;
pub mod file_watcher;
