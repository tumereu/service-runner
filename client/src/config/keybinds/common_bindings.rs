use crossterm::event::KeyCode;
use macros::PartialStruct;
use ui::input::KeyMatcher;
use crate::config::keybinds::keybinding::Keybinding;

#[derive(Debug, Clone, PartialStruct)]
pub struct CommonKeybindings {
    pub quit: Keybinding,
    pub focus_next: Keybinding,
    pub focus_prev: Keybinding,
    pub nav_left: Keybinding,
    pub nav_right: Keybinding,
    pub nav_up: Keybinding,
    pub nav_down: Keybinding,
    pub nav_left_large: Keybinding,
    pub nav_right_large: Keybinding,
    pub nav_up_large: Keybinding,
    pub nav_down_large: Keybinding,
    pub nav_to_start: Keybinding,
    pub nav_to_end: Keybinding,
    pub select: Keybinding,
    pub cancel: Keybinding,
}

impl Default for CommonKeybindings {
    fn default() -> Self {
        Self {
            quit: KeyMatcher::char('q').ctrl().into(),
            focus_next: KeyMatcher::new(KeyCode::Tab).into(),
            focus_prev: KeyMatcher::new(KeyCode::Tab).shift().into(),
            nav_left: [KeyMatcher::char('h'), KeyMatcher::new(KeyCode::Left)].into(),
            nav_left_large: [
                KeyMatcher::char('h').ctrl(),
                KeyMatcher::new(KeyCode::Left).ctrl(),
                KeyMatcher::char('h').shift(),
                KeyMatcher::new(KeyCode::Left).shift(),
            ]
            .into(),
            nav_right: [KeyMatcher::char('l'), KeyMatcher::new(KeyCode::Right)].into(),
            nav_right_large: [
                KeyMatcher::char('l').ctrl(),
                KeyMatcher::new(KeyCode::Right).ctrl(),
                KeyMatcher::char('l').shift(),
                KeyMatcher::new(KeyCode::Right).shift(),
            ]
            .into(),
            nav_up: [KeyMatcher::char('k'), KeyMatcher::new(KeyCode::Up)].into(),
            nav_up_large: [
                KeyMatcher::char('k').ctrl(),
                KeyMatcher::char('k').shift(),
                KeyMatcher::new(KeyCode::Up).ctrl(),
                KeyMatcher::new(KeyCode::Up).shift(),
                KeyMatcher::new(KeyCode::PageUp),
            ]
            .into(),
            nav_down: [KeyMatcher::char('j'), KeyMatcher::new(KeyCode::Down)].into(),
            nav_down_large: [
                KeyMatcher::char('j').ctrl(),
                KeyMatcher::char('j').shift(),
                KeyMatcher::new(KeyCode::Down).ctrl(),
                KeyMatcher::new(KeyCode::Down).shift(),
                KeyMatcher::new(KeyCode::PageDown),
            ]
            .into(),
            nav_to_start: KeyMatcher::char('g').into(),
            nav_to_end: KeyMatcher::char('g').shift().into(),
            select: [KeyMatcher::char(' '), KeyMatcher::new(KeyCode::Enter)].into(),
            cancel: KeyMatcher::new(KeyCode::Esc).into(),
        }
    }
}