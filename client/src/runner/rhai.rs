use std::sync::{mpsc, Arc, Mutex, PoisonError};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;
use log::error;
use crate::models::BlockStatus;
use crate::system_state::SystemState;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use rhai::{Dynamic, Engine, Map, Scope};
use rhai::plugin::RhaiResult;

pub struct RhaiExecutor {
    keep_alive: Arc<Mutex<bool>>,
    state: Arc<Mutex<SystemState>>,
    tx: Arc<Mutex<Option<Sender<(RhaiRequest, Sender<RhaiResult>)>>>>,
}
impl RhaiExecutor {
    pub fn new(state: Arc<Mutex<SystemState>>) -> Self {
        Self {
            keep_alive: Arc::new(Mutex::new(true)),
            state,
            tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let keep_alive = self.keep_alive.clone();
        let state_arc = self.state.clone();
        let (tx, rx) = channel::<(RhaiRequest, Sender<RhaiResult>)>();

        let worker_thread = thread::spawn(move || {
            let plain_engine = Self::init_rhai_engine();
            let function_engine = Self::init_rhai_engine();

            // FIXME add scripts

            while *keep_alive.lock().unwrap() {
                let query = rx.try_recv();
                match query {
                    Ok((request, response_tx)) => {
                        let engine = if request.allow_functions {
                            &function_engine
                        } else {
                            &plain_engine
                        };

                        let mut scope = Scope::new();
                        Self::populate_rhai_scope(state_arc.clone(), &mut scope, request.service_id);

                        let result = engine.eval_with_scope::<Dynamic>(&mut scope, &request.script);
                        match response_tx.send(result) {
                            Ok(_) => {}
                            Err(_) => {
                                error!("Failed to send response from Rhai worker thread.");
                            }
                        }
                    }
                    Err(_) => thread::sleep(std::time::Duration::from_millis(50))
                }
            }
        });

        self.tx = Arc::new(Mutex::new(Some(tx)));

        worker_thread
    }

    pub fn stop(&self) {
        *self.keep_alive.lock().unwrap() = false;
    }

    pub fn execute(&self, request: RhaiRequest) -> RhaiResult {
        let tx = self.tx.lock().unwrap();
        if let Some(tx) = tx.as_ref() {
            let (response_tx, response_rx) = channel::<RhaiResult>();

            tx.send((request, response_tx)).unwrap();
            response_rx.recv().unwrap()
        } else {
            // TODO custom error type instead?
            panic!("Failed to execute Rhai request. Rhai worker thread is not running.");
        }
    }

    fn populate_rhai_scope(
        state_arc: Arc<Mutex<SystemState>>,
        scope: &mut Scope,
        service_id: Option<String>,
    ) {
        let state = state_arc.lock().unwrap();

        state.iter_services().for_each(|service| {
            let mut blocks = Map::new();
            service.definition.blocks.iter().for_each(|block| {
                let mut block_map = Map::new();
                block_map.insert(
                    "status".into(),
                    match service.get_block_status(&block.id) {
                        BlockStatus::Initial => "Initial",
                        BlockStatus::Working { .. } => "Working",
                        BlockStatus::Ok => "Ok",
                        BlockStatus::Error => "Error",
                    }
                        .into(),
                );
                block_map.insert(
                    "is_processing".into(),
                    state.has_block_operations(&service.definition.id, &block.id).into(),
                );

                blocks.insert(block.id.clone().into(), block_map.into());
            });

            let mut service_map = Map::new();
            service_map.insert("blocks".into(), blocks.into());

            scope.push(service.definition.id.clone(), service_map.clone());
            match service_id.as_ref() {
                Some(service_id) if service_id == &service.definition.id => {
                    scope.push("self", service_map);
                }
                _ => {}
            }

            // Register helper constants to make it easier to check statuses
            scope.push_constant("INITIAL", "Initial");
            scope.push_constant("WORKING", "Working");
            scope.push_constant("OK", "Ok");
            scope.push_constant("ERROR", "Error");
        });
    }

    fn init_rhai_engine() -> Engine {
        let mut engine = Engine::new_raw();

        engine.set_max_strings_interned(1024);
        engine.set_module_resolver(DummyModuleResolver::new());
        engine.disable_symbol("eval");
        engine.disable_symbol("print");
        engine.disable_symbol("debug");
        engine.disable_symbol("import");

        let std_package = StandardPackage::new();
        std_package.register_into_engine(&mut engine);

        engine
    }
}

pub struct RhaiRequest {
    pub script: String,
    pub allow_functions: bool,
    pub service_id: Option<String>,
}

