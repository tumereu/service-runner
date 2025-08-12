mod key_matcher;

pub use key_matcher::*;

use crossterm::event::Event;
use std::time::Duration;

pub fn collect_input_events() -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    
    while crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let event = crossterm::event::read().unwrap();
        events.push(event);
    }
    
    events
}