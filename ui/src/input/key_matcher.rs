use crate::Signals;
use crossterm::event::{KeyCode, KeyEventKind};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct KeyMatcher {
    key: KeyCode,
    shift: bool,
    ctrl: bool,
    alt: bool,
}
impl KeyMatcher {
    pub fn char(char: char) -> Self {
        Self::new(KeyCode::Char(char.to_ascii_lowercase()))
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
        let key_matches = match (event.code, self.key) {
            (KeyCode::Char(ev_char), KeyCode::Char(req_char)) => ev_char.to_ascii_lowercase() == req_char,
            (KeyCode::BackTab, KeyCode::Tab) if self.shift => true,
            _ => event.code == self.key,
        };
        key_matches
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
    fn is_key_pressed<B : Borrow<Vec<KeyMatcher>>>(&self, matcher: B) -> bool;
}
impl KeyMatcherQueryable for Signals {
    fn is_key_pressed<B : Borrow<Vec<KeyMatcher>>>(&self, matcher: B) -> bool {
        self.matching::<crossterm::event::KeyEvent>()
            .iter()
            .any(|ev| {
                ev.kind == KeyEventKind::Press && matcher.borrow().iter().any(|matcher| matcher.matches(ev))
            })
    }
}

impl Serialize for KeyMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = String::new();

        if self.shift {
            s.push_str("<Shift>");
        }
        if self.ctrl {
            s.push_str("<Ctrl>");
        }
        if self.alt {
            s.push_str("<Alt>");
        }

        match &self.key {
            KeyCode::Backspace => s.push_str("<Backspace>"),
            KeyCode::Enter => s.push_str("<Enter>"),
            KeyCode::Esc => s.push_str("<Esc>"),
            KeyCode::Left => s.push_str("<Left>"),
            KeyCode::Right => s.push_str("<Right>"),
            KeyCode::Up => s.push_str("<Up>"),
            KeyCode::Down => s.push_str("<Down>"),
            KeyCode::Tab => s.push_str("<Tab>"),
            KeyCode::Null => s.push_str("<Null>"),
            KeyCode::BackTab => s.push_str("<BackTab>"),
            KeyCode::Insert => s.push_str("<Insert>"),
            KeyCode::Delete => s.push_str("<Delete>"),
            KeyCode::Home => s.push_str("<Home>"),
            KeyCode::End => s.push_str("<End>"),
            KeyCode::PageUp => s.push_str("<PageUp>"),
            KeyCode::PageDown => s.push_str("<PageDown>"),
            KeyCode::F(n) => s.push_str(&format!("<F{}>", n)),
            KeyCode::CapsLock => s.push_str("<CapsLock>"),
            KeyCode::ScrollLock => s.push_str("<ScrollLock>"),
            KeyCode::NumLock => s.push_str("<NumLock>"),
            KeyCode::Pause => s.push_str("<Pause>"),
            KeyCode::PrintScreen => s.push_str("<PrintScr>"),
            KeyCode::Menu => s.push_str("<Menu>"),
            KeyCode::Char(c) => s.push(*c),
            _ => panic!("Unrecognized keycode: {:?}", self.key),
        }

        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for KeyMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut remaining = s.as_str();
        let mut shift = false;
        let mut ctrl = false;
        let mut alt = false;

        // Loop through modifiers in any order
        loop {
            if let Some(start) = remaining.find('<') {
                if let Some(end) = remaining[start + 1..].find('>') {
                    let tag = &remaining[start + 1..start + 1 + end];
                    let tag_lower = tag.to_lowercase();

                    match tag_lower.as_str() {
                        "shift" => shift = true,
                        "ctrl" => ctrl = true,
                        "alt" => alt = true,
                        _ => break, // unknown tag -> stop parsing modifiers
                    }

                    // Remove the parsed modifier
                    remaining = &remaining[start + 1 + end + 1..];
                    continue;
                }
            }
            break;
        }

        // Match key, case-insensitive
        let key_lower = remaining.to_lowercase();
        let key = match key_lower.as_str() {
            "<backspace>" => KeyCode::Backspace,
            "<enter>" => KeyCode::Enter,
            "<esc>" => KeyCode::Esc,
            "<left>" => KeyCode::Left,
            "<right>" => KeyCode::Right,
            "<up>" => KeyCode::Up,
            "<down>" => KeyCode::Down,
            "<tab>" => KeyCode::Tab,
            "<null>" => KeyCode::Null,
            "<backtab>" => KeyCode::BackTab,
            "<insert>" => KeyCode::Insert,
            "<delete>" => KeyCode::Delete,
            "<home>" => KeyCode::Home,
            "<end>" => KeyCode::End,
            "<pageup>" | "<pgup>"=> KeyCode::PageUp,
            "<pagedown>" | "<pgdown> "=> KeyCode::PageDown,
            "<f1>" => KeyCode::F(1),
            "<f2>" => KeyCode::F(2),
            "<f3>" => KeyCode::F(3),
            "<f4>" => KeyCode::F(4),
            "<f5>" => KeyCode::F(5),
            "<f6>" => KeyCode::F(6),
            "<f7>" => KeyCode::F(7),
            "<f8>" => KeyCode::F(8),
            "<f9>" => KeyCode::F(9),
            "<f10>" => KeyCode::F(10),
            "<f11>" => KeyCode::F(11),
            "<f12>" => KeyCode::F(12),
            "<capslock>" => KeyCode::CapsLock,
            "<scrolllock>" => KeyCode::ScrollLock,
            "<numlock>" => KeyCode::NumLock,
            "<pause>" => KeyCode::Pause,
            "<printscr>" | "<printscreen>" => KeyCode::PrintScreen,
            "<menu>" => KeyCode::Menu,
            s if s.chars().count() == 1 => KeyCode::Char(s.chars().next().unwrap()),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Unrecognized key: {}",
                    remaining
                )))
            }
        };

        Ok(KeyMatcher {
            key,
            shift,
            ctrl,
            alt,
        })
    }
}