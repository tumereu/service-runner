#[derive(Eq, PartialEq)]
pub enum UIState {
    Initializing,
    ProfileSelect {
        selected_idx: usize
    }
}