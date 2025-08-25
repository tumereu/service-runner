use crate::config::{BlockId, ServiceId, TaskDefinitionId};
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::system_state::SystemState;
use log::error;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, Engine, Map, Scope};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use crate::runner::scripting::engine::ScriptEngine;

pub struct ScriptExecutor {
    keep_alive: Arc<Mutex<bool>>,
    state: Arc<RwLock<SystemState>>,
    tx: Arc<Mutex<Option<Sender<(RhaiRequest, Sender<RhaiResult>)>>>>,
}
impl ScriptExecutor {
    pub fn new(state: Arc<RwLock<SystemState>>) -> Self {
        Self {
            keep_alive: Arc::new(Mutex::new(true)),
            state,
            tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let keep_alive = self.keep_alive.clone();
        let state_arc = self.state.clone();
        let (tx, rx) = channel::<(RhaiRequest, Sender<RhaiResult>)>();

        let worker_thread = thread::spawn(move || {
            let mut plain_engine = ScriptEngine::new(state_arc.clone(), false);
            let mut function_engine = ScriptEngine::new(state_arc.clone(), true);

            while *keep_alive.lock().unwrap() {
                let query = rx.try_recv();
                match query {
                    Ok((request, response_tx)) => {
                        let engine = if request.allow_functions {
                            &mut function_engine
                        } else {
                            &mut plain_engine
                        };

                        engine.set_self_service(&request.service_id);
                        let result = engine.eval(&request.script);
                        match response_tx.send(result) {
                            Ok(_) => {}
                            Err(_) => {
                                error!("Failed to send response from Rhai worker thread.");
                            }
                        }
                    }
                    Err(_) => thread::sleep(std::time::Duration::from_millis(50)),
                }
            }
        });

        *self.tx.lock().unwrap() = Some(tx);

        worker_thread
    }

    pub fn stop(&self) {
        *self.keep_alive.lock().unwrap() = false;
    }

    pub fn enqueue(&self, request: RhaiRequest) -> Receiver<RhaiResult> {
        let tx = self.tx.lock().unwrap();
        if let Some(tx) = tx.as_ref() {
            let (response_tx, response_rx) = channel::<RhaiResult>();

            tx.send((request, response_tx)).unwrap();
            response_rx
        } else {
            // TODO custom error type instead?
            panic!("Failed to execute Rhai request. Rhai worker thread is not running.");
        }
    }
}

pub struct RhaiRequest {
    pub script: String,
    pub allow_functions: bool,
    pub service_id: Option<ServiceId>,
}

#[derive(Clone)]
struct ServiceProxy {
    id: String,
}

#[derive(Clone)]
struct BlockProxy {
    service_id: String,
    block_id: String,
}
