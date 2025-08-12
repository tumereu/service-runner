use std::borrow::Borrow;
use crate::{Signals};
use crossterm::event::{KeyCode, KeyEventKind, KeyEventState};

pub struct KeyMatcher {
    key: KeyCode,
    shift: bool,
    ctrl: bool,
    alt: bool,
}
impl KeyMatcher {
    pub fn char(char: char) -> Self {
        Self::new(KeyCode::Char(char))
    }

    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    pub fn shift(mut self) -> KeyMatcher {
        self.shift = true;
        self
    }

    pub fn ctrl(mut self) -> KeyMatcher {
        self.ctrl = true;
        self
    }

    pub fn alt(mut self) -> KeyMatcher {
        self.alt = true;
        self
    }

    pub fn matches(&self, event: &crossterm::event::KeyEvent) -> bool {
        self.key == event.code
            && self.shift
                == event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
            && self.ctrl
                == event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
            && self.alt
                == event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::ALT)
    }

    pub fn matches_event(&self, event: &crossterm::event::Event) -> bool {
        match event {
            crossterm::event::Event::Key(key) => self.matches(key),
            _ => false,
        }
    }
}

pub trait KeyMatcherQueryable {
    fn is_key_pressed<B : Borrow<KeyMatcher>>(&self, matcher: B) -> bool;
}
impl KeyMatcherQueryable for Signals {
    fn is_key_pressed<B : Borrow<KeyMatcher>>(&self, matcher: B) -> bool {
        self.matching::<crossterm::event::KeyEvent>()
            .iter()
            .any(|ev| {
                ev.kind == KeyEventKind::Press && matcher.borrow().matches(ev)
            })
    }
}
