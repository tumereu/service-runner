#[derive(Eq, PartialEq, Debug)]
pub enum UIState {
    Initializing,
    ProfileSelect {
        selected_idx: usize
    }
}