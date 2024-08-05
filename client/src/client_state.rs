use std::collections::VecDeque;

use crate::model::config::Config;
use crate::model::message::models::OutputStore;
use crate::model::message::{Action, Broadcast};
use crate::model::system_state::SystemState;

use crate::ui::UIState;

pub struct ClientState {
    pub status: ClientStatus,
    pub system_state: Option<SystemState>,
    pub actions_out: VecDeque<Action>,
    pub broadcasts_in: VecDeque<Broadcast>,
    pub output_store: OutputStore,
    pub ui: UIState,
    pub config: Config,
    pub last_frame_size: (u16, u16)
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        ClientState {
            status: ClientStatus::Ready,
            system_state: None,
            actions_out: VecDeque::new(),
            broadcasts_in: VecDeque::new(),
            ui: UIState::Initializing,
            output_store: OutputStore::new(),
            last_frame_size: (0, 0),
            config,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ClientStatus {
    Ready,
    Exiting,
}
