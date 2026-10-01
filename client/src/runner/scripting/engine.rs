use crate::config::{BlockId, ServiceId, TaskDefinitionId};
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::system_state::SystemState;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, Engine, Map, Scope};
use std::sync::{Arc, RwLock};

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
    scope_len: usize,
}
impl ScriptEngine {
    pub fn new(state: Arc<RwLock<SystemState>>, with_fn: bool) -> Self {
        let mut engine = Self::init_rhai_engine();
        let scope = Self::init_rhai_scope(&state.read().unwrap());
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
        // `self` is the last cached binding; replace it without rebuilding the services map.
        self.scope.rewind(self.scope_len - 1);
        match service_id {
            Some(service_id) => self.scope.push_constant(
                "self",
                Dynamic::from(ServiceProxy {
                    id: service_id.inner().to_owned(),
                }),
            ),
            None => self.scope.push_constant("self", Dynamic::from(())),
        };
    }

    pub fn eval(&mut self, script: &str) -> RhaiResult {
        // Reuse the cached service proxies and constants; discard only script-local bindings
        // left by the previous evaluation, including one that failed. The proxies read live
        // service state from SystemState, which is not affected by rewinding the scope.
        self.scope.rewind(self.scope_len);
        self.engine
            .eval_with_scope::<Dynamic>(&mut self.scope, script)
    }

    fn init_rhai_scope(state: &SystemState) -> Scope<'static> {
        // Service membership is fixed after profile loading. Building this cached map earlier
        // would permanently capture an empty set, even though proxy state reads remain live.
        assert!(
            state.current_profile.is_some(),
            "Initialize Rhai only after loading a profile"
        );
        let mut scope = Scope::new();
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

        // Keep this last so changing the current service only replaces one binding.
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
        engine.register_get("id", |self_proxy: &mut ServiceProxy| {
            self_proxy.id.to_owned()
        });
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
                        BlockStatus::Ok { .. } => "Ok",
                        BlockStatus::Error => "Error",
                    }
                } else {
                    "Unknown"
                }
            });
        }
        {
            let state = state.clone();
            engine.register_get("work_skipped", move |blk: &mut BlockProxy| {
                let state = state.read().unwrap();
                if let Some(service) = state.get_service(&ServiceId::new(&blk.service_id)) {
                    match service.get_block_status(&BlockId::new(&blk.block_id)) {
                        BlockStatus::Ok { was_worked } => !was_worked,
                        _ => false,
                    }
                } else {
                    false
                }
            });
        }

        {
            let state = state.clone();
            engine.register_get("is_processing", move |blk: &mut BlockProxy| {
                let state = state.read().unwrap();
                state.has_block_operations(
                    &ServiceId::new(&blk.service_id),
                    &BlockId::new(&blk.block_id),
                )
            });
        }

        {
            let state = state.clone();
            engine.register_get("is_idle", move |blk: &mut BlockProxy| {
                let state = state.read().unwrap();
                state
                    .query_service(&ServiceId::new(&blk.service_id), |service| {
                        let block_id = BlockId::new(&blk.block_id);
                        if service.get_block_action(&block_id).is_some() {
                            false
                        } else {
                            match service.get_block_status(&block_id) {
                                BlockStatus::Working { .. } => false,
                                _ => true,
                            }
                        }
                    })
                    .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Settings};

    fn state() -> Arc<RwLock<SystemState>> {
        let service = serde_yaml::from_str(
            r#"
id: some-library
workdir: .
blocks:
  - id: build
    type: cmd-seq
    commands: []
    status_line: {symbol: C, slot: 20}
"#,
        )
        .unwrap();
        let profile =
            serde_yaml::from_str("id: test\nworkdir: .\nservices: [{id: some-library}]").unwrap();
        let empty_profile = serde_yaml::from_str("id: empty\nworkdir: .\nservices: []").unwrap();
        Arc::new(RwLock::new(SystemState::new(
            Config {
                conf_dir: ".".into(),
                settings: Settings::default(),
                services: vec![service],
                profiles: vec![profile, empty_profile],
            },
            ".".into(),
        )))
    }

    #[test]
    fn executor_handles_manual_and_automatic_profile_selection() {
        use crate::runner::scripting::executor::{RhaiRequest, ScriptExecutor};
        use std::{thread, time::Duration};

        for autolaunch in [false, true] {
            let state = state();
            if autolaunch {
                state.write().unwrap().select_profile("test");
            }
            let executor = ScriptExecutor::new(state.clone());
            let handle = executor.start();
            if !autolaunch {
                // Exercise time spent in the menu with the executor thread already running.
                thread::sleep(Duration::from_millis(100));
            }
            {
                let mut state = state.write().unwrap();
                if !autolaunch {
                    state.select_profile("test");
                }
                state.update_service(&ServiceId::new("some-library"), |service| {
                    service.update_block_action(&BlockId::new("build"), None);
                    service.update_block_status(
                        &BlockId::new("build"),
                        BlockStatus::Ok { was_worked: true },
                    );
                });
            }

            let results: Vec<_> = [false, true].into_iter().map(|allow_functions| {
                executor.enqueue(RhaiRequest {
                    script: "services[\"some-library\"].blocks.build.is_idle && services[\"some-library\"].blocks.build.status == OK".into(),
                    allow_functions,
                    service_id: None,
                }).recv_timeout(Duration::from_secs(2))
            }).collect();
            executor.stop();
            handle.join().unwrap();
            for result in results {
                assert!(result.unwrap().unwrap().as_bool().unwrap());
            }
        }
    }

    #[test]
    fn cached_scope_reads_live_service_state() {
        let state = state();
        state.write().unwrap().select_profile("test");
        let mut engine = ScriptEngine::new(state.clone(), false);
        assert_eq!(engine.eval("services.len()").unwrap().as_int().unwrap(), 1);
        assert!(
            engine
                .eval("services[\"some-library\"].blocks.build.status == INITIAL")
                .unwrap()
                .as_bool()
                .unwrap()
        );
        state
            .write()
            .unwrap()
            .update_service(&ServiceId::new("some-library"), |service| {
                service.update_block_status(
                    &BlockId::new("build"),
                    BlockStatus::Ok { was_worked: true },
                );
            });
        assert!(
            engine
                .eval("services[\"some-library\"].blocks.build.status == OK")
                .unwrap()
                .as_bool()
                .unwrap()
        );
    }

    #[test]
    fn self_and_actions_work_with_cached_scope() {
        let state = state();
        state.write().unwrap().select_profile("test");
        let mut engine = ScriptEngine::new(state.clone(), true);
        engine.set_self_service(&Some(ServiceId::new("some-library")));
        let _ = engine.eval("disable(self.id, \"build\")").unwrap();
        assert!(matches!(
            state
                .read()
                .unwrap()
                .get_service(&ServiceId::new("some-library"))
                .unwrap()
                .get_block_action(&BlockId::new("build")),
            Some(BlockAction::Disable)
        ));
        assert!(
            engine
                .eval("self.id == services[\"some-library\"].id")
                .unwrap()
                .as_bool()
                .unwrap()
        );
        engine.set_self_service(&None);
        assert!(engine.eval("self == ()").unwrap().as_bool().unwrap());
    }

    #[test]
    fn cached_scope_discards_script_locals_after_success_and_failure() {
        let state = state();
        state.write().unwrap().select_profile("test");
        let mut engine = ScriptEngine::new(state, false);
        assert_eq!(
            engine
                .eval("let local = 42; local")
                .unwrap()
                .as_int()
                .unwrap(),
            42
        );
        assert!(engine.eval("local").is_err());
        assert!(
            engine
                .eval("let failed_local = 42; throw \"error\";")
                .is_err()
        );
        assert!(engine.eval("failed_local").is_err());
        assert_eq!(engine.eval("services.len()").unwrap().as_int().unwrap(), 1);
    }

    #[test]
    fn workers_can_stop_while_waiting_in_the_main_menu() {
        use crate::runner::scripting::executor::ScriptExecutor;
        use crate::runner::service_worker::ServiceWorker;

        let state = state();
        let executor = Arc::new(ScriptExecutor::new(state.clone()));
        let worker = ServiceWorker::new(state, executor.clone());
        let executor_handle = executor.start();
        let worker_handle = worker.start();
        executor.stop();
        worker.stop();
        executor_handle.join().unwrap();
        worker_handle.join().unwrap();
    }
}
