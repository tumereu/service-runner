use crate::ui::state::ViewProfilePane::ServiceList;

#[derive(Eq, PartialEq, Debug)]
pub enum UIState {
    Initializing,
    Exiting,
    ProfileSelect {
        selected_idx: usize,
    },
    ViewProfile {
        active_pane: ViewProfilePane,
        service_selection: usize,
    },
}
impl UIState {
    pub fn profile_select() -> UIState {
        UIState::ProfileSelect { selected_idx: 0 }
    }

    pub fn view_profile() -> UIState {
        UIState::ViewProfile {
            active_pane: ServiceList,
            service_selection: 0,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ViewProfilePane {
    ServiceList,
    OutputPane,
}
