use std::collections::HashMap;
use crate::config::{Block, BlockId, ServiceId, TaskDefinitionId};
use crate::models::{BlockAction, BlockStatus, Service, WorkStep};
use crate::system_state::SystemState;
use crossterm::style::Stylize;
use log::error;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, Engine, Map, Scope};
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError, RwLock, mpsc};
use std::thread;
use std::thread::JoinHandle;

pub struct RhaiExecutor {
    keep_alive: Arc<Mutex<bool>>,
    state: Arc<RwLock<SystemState>>,
    tx: Arc<Mutex<Option<Sender<(RhaiRequest, Sender<RhaiResult>)>>>>,
}
impl RhaiExecutor {
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
            let mut plain_engine = Self::init_rhai_engine();
            Self::register_proxies(state_arc.clone(), &mut plain_engine);

            let mut function_engine = Self::init_rhai_engine();
            Self::register_functions(state_arc.clone(), &mut function_engine);
            Self::register_proxies(state_arc.clone(), &mut function_engine);

            let mut scope = Self::init_rhai_scope(state_arc.clone());
            let mut scope_len = scope.len();

            while *keep_alive.lock().unwrap() {
                let query = rx.try_recv();
                match query {
                    Ok((request, response_tx)) => {
                        let engine = if request.allow_functions {
                            &function_engine
                        } else {
                            &plain_engine
                        };

                        scope.rewind(scope_len);
                        if let Some(service_id) = request.service_id {
                            scope.push_constant("self", Dynamic::from(ServiceProxy { id: service_id.inner().to_owned() }));
                        }

                        let result = engine.eval_with_scope::<Dynamic>(&mut scope, &request.script);
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

    fn init_rhai_scope<'a>(
        state_arc: Arc<RwLock<SystemState>>,
    ) -> Scope<'a> {
        let mut scope = Scope::<'a>::new();
        let state = state_arc.read().unwrap();
        let mut services_map = Map::new();

        state.iter_services().for_each(|service| {
            let id = service.definition.id.inner().to_owned();
            services_map.insert(id.clone().into(), Dynamic::from(ServiceProxy { id }));
        });

        scope.push_constant("services", services_map);

        // Register helper constants to make it easier to check statuses
        scope.push_constant("INITIAL", "Initial");
        scope.push_constant("DISABLED", "Disabled");
        scope.push_constant("WAITING", "Waiting");
        scope.push_constant("WORKING", "Working");
        scope.push_constant("OK", "Ok");
        scope.push_constant("ERROR", "Error");

        scope
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

    fn register_functions(state_arc: Arc<RwLock<SystemState>>, function_engine: &mut Engine) {
        [
            ("disable", BlockAction::Disable),
            ("enable", BlockAction::Enable),
            ("toggle", BlockAction::ToggleEnabled),
            ("run", BlockAction::Run),
            ("rerun", BlockAction::ReRun),
            ("stop", BlockAction::Stop),
            ("cancel", BlockAction::Cancel),
        ]
        .into_iter()
        .for_each(|(name, action)| {
            let state_arc = state_arc.clone();
            function_engine.register_fn(name, move |service: &str, block: &str| {
                let mut state = state_arc.write().unwrap();
                state.update_service(&ServiceId::new(service), |service| {
                    service.update_block_action(&BlockId::new(block), Some(action.clone()))
                });
            });
        });

        {
            let state_arc = state_arc.clone();
            function_engine.register_fn("spawn_task", move |service: &str, definition_id: &str| {
                let mut state = state_arc.write().unwrap();
                state.current_profile.iter_mut().for_each(|profile| {
                    profile.spawn_task(
                        &TaskDefinitionId(definition_id.to_owned()),
                        Some(ServiceId::new(service)),
                    );
                });
            });
        }
        {
            let state_arc = state_arc.clone();
            function_engine.register_fn("spawn_task", move |definition_id: &str| {
                let mut state = state_arc.write().unwrap();
                state.current_profile.iter_mut().for_each(|profile| {
                    profile.spawn_task(&TaskDefinitionId(definition_id.to_owned()), None);
                });
            });
        }
    }

    fn register_proxies(state: Arc<RwLock<SystemState>>, engine: &mut Engine) {
        engine.register_type_with_name::<ServiceProxy>("Service");
        engine.register_type_with_name::<BlockProxy>("Block");

        // Service proxy properties
        engine.register_get("id", |self_proxy: &mut ServiceProxy| self_proxy.id.to_owned());
        {
            let state = state.clone();
            engine.register_get("blocks", move |svc: &mut ServiceProxy| {
                let state = state.read().unwrap();
                let mut map = Map::new();

                if let Some(service) = state.get_service(&ServiceId::new(&svc.id)) {
                    for block in &service.definition.blocks {
                        let proxy = BlockProxy {
                            service_id: svc.id.clone(),
                            block_id: block.id.inner().to_owned(),
                        };
                        map.insert(block.id.inner().into(), Dynamic::from(proxy));
                    }
                }

                map
            });
        }

        // Block proxy properties
        engine.register_get("id", |blk: &mut BlockProxy| blk.block_id.to_owned());
        {
            let state = state.clone();
            engine.register_get("status", move |blk: &mut BlockProxy| {
                let state = state.read().unwrap();
                if let Some(service) = state.get_service(&ServiceId::new(&blk.service_id)) {
                    match service.get_block_status(&BlockId::new(&blk.block_id)) {
                        BlockStatus::Disabled => "Disabled",
                        BlockStatus::Initial => "Initial",
                        BlockStatus::Working {
                            step: WorkStep::ResourceGroupCheck { .. },
                        } => "Waiting",
                        BlockStatus::Working {
                            step: WorkStep::PrerequisiteCheck { last_failure, .. },
                        } if last_failure.is_some() => "Waiting",
                        BlockStatus::Working { .. } => "Working",
                        BlockStatus::Ok => "Ok",
                        BlockStatus::Error => "Error",
                    }
                } else {
                    "Unknown"
                }
            });
        }

        {
            let state = state.clone();
            engine.register_get("is_processing", move |blk: &mut BlockProxy| {
                let state = state.read().unwrap();
                state.has_block_operations(&ServiceId::new(&blk.service_id), &BlockId::new(&blk.block_id))
            });
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
