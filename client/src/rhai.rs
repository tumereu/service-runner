use rhai::{Scope, Map, Engine};
use once_cell::sync::Lazy;
use rhai::module_resolvers::DummyModuleResolver;
use rhai::packages::{Package, StandardPackage};
use crate::models::BlockStatus;
use crate::system_state::SystemState;

pub fn populate_rhai_scope(scope: &mut Scope, state: &SystemState, service_id: &str) {
    let mut scope = Scope::new();

    state.iter_services()
        .for_each(|service| {
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
                    }.into(),
                );
                block_map.insert(
                    "is_processing".into(),
                    state.has_block_operations(service_id, &block.id).into()
                );

                blocks.insert(block.id.clone().into(), block_map.into());
            });

            let mut service_map = Map::new();
            service_map.insert("blocks".into(), blocks.into());

            scope.push(service.definition.id.clone(), service_map.clone());
            if service.definition.id == *service_id {
                scope.push("self", service_map);
            }

            // Register helper constants to make it easier to check statuses
            scope.push_constant("INITIAL", "Initial");
            scope.push_constant("WORKING", "Working");
            scope.push_constant("OK", "Ok");
            scope.push_constant("ERROR", "Error");
        });
}

pub const RHAI_ENGINE: Lazy<Engine> = Lazy::new(|| {
    let mut engine = Engine::new_raw();

    engine.set_max_strings_interned(1024);
    engine.set_module_resolver(DummyModuleResolver::new());

    let std_package = StandardPackage::new();
    std_package.register_into_engine(&mut engine);

    engine
});