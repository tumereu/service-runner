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
        wrap_output: bool,
        output_pos_vert: Option<u128>,
        output_pos_horiz: Option<u64>
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
            wrap_output: false,
            output_pos_horiz: None,
            output_pos_vert: None,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ViewProfilePane {
    ServiceList,
    OutputPane,
}
