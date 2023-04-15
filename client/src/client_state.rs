use shared::config::Config;
use shared::message::{Action, Broadcast};
use shared::message::models::OutputStore;
use shared::system_state::SystemState;

use crate::ui::UIState;

pub struct ClientState {
    pub status: ClientStatus,
    pub system_state: Option<SystemState>,
    pub actions_out: Vec<Action>,
    pub broadcasts_in: Vec<Broadcast>,
    pub output_store: OutputStore,
    pub ui: UIState,
    pub config: Config,
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        ClientState {
            status: ClientStatus::Ready,
            system_state: None,
            actions_out: Vec::new(),
            broadcasts_in: Vec::new(),
            ui: UIState::Initializing,
            output_store: OutputStore::new(),
            config,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ClientStatus {
    Ready,
    Exiting,
}