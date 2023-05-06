#[derive(Debug)]
pub enum UIState {
    Initializing,
    Exiting,
    ProfileSelect {
        selected_idx: usize,
    },
    ViewProfile(ViewProfileState),
}
impl UIState {
    pub fn profile_select() -> UIState {
        UIState::ProfileSelect { selected_idx: 0 }
    }

    pub fn view_profile() -> UIState {
        UIState::ViewProfile(ViewProfileState {
            active_pane: ViewProfilePane::ServiceList,
            service_selection: 0,
            wrap_output: false,
            output_pos_horiz: None,
            output_pos_vert: None,
        })
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ViewProfilePane {
    ServiceList,
    OutputPane,
}

#[derive(Debug, Clone)]
pub struct ViewProfileState {
    pub active_pane: ViewProfilePane,
    pub service_selection: usize,
    pub wrap_output: bool,
    pub output_pos_vert: Option<u128>,
    pub output_pos_horiz: Option<u64>
}