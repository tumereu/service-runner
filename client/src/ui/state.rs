#[derive(Eq, PartialEq, Debug)]
pub enum UIState {
    Initializing,
    ProfileSelect {
        selected_idx: usize
    },
    ViewProfile {

    }
}
impl UIState {
    pub fn profile_select() -> UIState {
        UIState::ProfileSelect {
            selected_idx: 0
        }
    }

    pub fn view_profile() -> UIState {
        UIState::ViewProfile {}
    }
}