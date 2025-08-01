use crate::models::{BlockStatus, Service};
use crate::system_state::SystemState;
use std::sync::{Arc, Mutex};

pub struct BlockWorker {
    pub state_arc: Arc<Mutex<SystemState>>,
    pub service_id: String,
    pub block_id: String,
}
impl BlockWorker {
    fn clear_current_action(&self) {
        let mut state = self.state_arc.lock().unwrap();
        state.update_service(&self.service_id, |service| {
            service.update_block_action(&self.block_id, None)
        })
    }

    fn update_status(&self, status: BlockStatus) {
        let mut state = self.state_arc.lock().unwrap();
        state.update_service(&self.service_id, |service| {
            service.update_block_status(&self.block_id, status)
        });
    }
    
}
