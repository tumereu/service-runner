use crate::config::{BlockId, ServiceId, TaskDefinitionId};
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::system_state::SystemState;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, Engine, Map, Scope};
use std::sync::{Arc, Mutex, RwLock};

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
    scope_len: usize,
}
impl ScriptEngine {
    pub fn new(
        state: Arc<RwLock<SystemState>>,
        with_fn: bool,
    ) -> Self {
        let mut engine = Self::init_rhai_engine();
        let scope = Self::init_rhai_scope(state.clone());
        
        Self::register_proxies(state.clone(), &mut engine);
        if with_fn {
            Self::register_functions(state.clone(), &mut engine);
        }
        
        Self {
            engine,
            scope_len: scope.len(),
            scope,
        }
    }
    
    pub fn set_self_service(&mut self, service_id: &Option<ServiceId>) {
        self.scope.rewind(self.scope_len - 1);
        match service_id { 
            Some(service_id) => self.scope.push_constant("self", Dynamic::from(ServiceProxy { id: service_id.inner().to_owned() })),
            None => self.scope.push_constant("self", Dynamic::from(())),
        };
    }
    
    pub fn eval(&mut self, script: &str) -> RhaiResult {
        self.scope.rewind(self.scope_len);
        self.engine.eval_with_scope::<Dynamic>(&mut self.scope, script)
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

        // Push the self-constant last, so that rewinding to scope.len() - 1 will remove it.
        scope.push_constant("self", Dynamic::from(()));

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
