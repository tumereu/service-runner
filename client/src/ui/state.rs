use crate::ui::CurrentScreen::ProfileSelect;

#[derive(Debug)]
pub struct UIState {
    pub last_frame_size: (u16, u16),
    pub screen: CurrentScreen
}
impl UIState {
    pub fn new() -> UIState {
        UIState {
            last_frame_size: (0, 0),
            screen: ProfileSelect {
                selected_idx: 0
            }
        }
    }
}

#[derive(Debug)]
pub enum CurrentScreen {
    ProfileSelect {
        selected_idx: usize,
    },
    ViewProfile(ViewProfileState),
}
impl CurrentScreen {
    pub fn profile_select() -> CurrentScreen {
        CurrentScreen::ProfileSelect { selected_idx: 0 }
    }

    pub fn view_profile() -> CurrentScreen {
        CurrentScreen::ViewProfile(ViewProfileState {
            active_pane: ViewProfilePane::ServiceList,
            service_selection: 0,
            wrap_output: false,
            output_pos_horiz: None,
            output_pos_vert: None,
            floating_pane: None,
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
    pub output_pos_horiz: Option<u64>,
    pub floating_pane: Option<ViewProfileFloatingPane>,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ViewProfileFloatingPane {
    ServiceAutocompleteDetails {
        detail_list_selection: usize
    }
}
