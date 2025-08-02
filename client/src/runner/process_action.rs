use crate::models::Action;
use crate::system_state::SystemState;
use crate::ui::CurrentScreen;

pub fn process_action(system: &mut SystemState, action: Action) {
    match action {
        Action::Shutdown => {
            system.should_exit = true;
        },
        Action::ActivateProfile(profile) => {
            system.current_profile = Some(profile);
            system.ui.screen = CurrentScreen::view_profile();
        },
        _ => {
            // TODO fix
        }
    }
}
