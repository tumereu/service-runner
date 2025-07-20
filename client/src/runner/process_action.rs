use std::time::Instant;
use crate::models::{Action};

use crate::runner::automation::process_pending_automations;
use crate::system_state::SystemState;
use crate::ui::CurrentScreen;

pub fn process_action(system: &mut SystemState, action: Action) {
    match action {
        Action::Shutdown => {
            system.should_exit = true;
        },
        Action::ActivateProfile(profile) => {
            system.update_state(|state| {
                state.current_profile = Some(profile);
                state.ui.screen = CurrentScreen::view_profile();
            });
        },
        _ => {
            // TODO fix
        }
        Action::Reset(_service_name) => {
            // TODO implement
        },
        Action::ResetAll => {
            // TODO implement
        }
    }
}
