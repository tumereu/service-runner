use std::cell::RefCell;
use crate::config::{BlockId, ServiceId};
use crate::models::BlockAction;
use crate::system_state::SystemState;

pub enum Action {
    SelectProfile(String),
    ToggleOutput(ServiceId),
    ToggleOutputAll,
    SetBlockAction(ServiceId, BlockId, BlockAction),
}

pub struct ActionStore(RefCell<Vec<Action>>);
impl ActionStore {
    pub fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }
    
    pub fn register(&self, action: Action) {
        self.0.borrow_mut().push(action);
    }
    
    pub fn process(self, state: &mut SystemState) {
        self.0.take().into_iter().for_each(|action| Self::process_action(action, state));
    }
    
    fn process_action(action: Action, state: &mut SystemState) {
        match action { 
            Action::SelectProfile(profile_id) => state.select_profile(&profile_id),
            Action::ToggleOutput(service_id) => state.update_service(&service_id, |service| {
                service.output_enabled = !service.output_enabled;
            }),
            Action::ToggleOutputAll => state.update_all_services(|service| {
                service.output_enabled = !service.output_enabled;
            }),
            Action::SetBlockAction(service_id, block_id, action) => {
                state.update_service(&service_id, |service| {
                    service.update_block_action(&block_id, Some(action));
                })
            }
        }
    }
}