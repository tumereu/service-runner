use crate::component::{Component, Text, ATTR_COLOR_HIGHLIGHT, ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_LEFT, ATTR_KEY_NAV_RIGHT, ATTR_KEY_NAV_UP};
use crate::frame_ctx::{FrameContext, RenderArgs};
use crate::{SignalHandling, Signals, UIError, UIResult};
use ratatui::backend::Backend;
use ratatui::style::Color;
use ratatui::Terminal;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crossterm::event::KeyCode;
use crate::input::KeyMatcher;

pub struct ComponentRenderer {
    states: HashMap<String, Box<dyn Any>>,
    signals: Signals,
    attributes: HashMap<String, Box<dyn Any>>,
}
impl ComponentRenderer {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            signals: Signals::empty(),
            attributes: HashMap::new(),
        }
    }

    pub fn send_signal<T>(&mut self, signal:  T) where T : Any + 'static {
        self.signals.push(signal);
    }

    pub fn send_input_signals(&mut self, inputs: Vec<crossterm::event::Event>) {
        for input in inputs {
            match input {
                crossterm::event::Event::Key(key) => {
                    self.send_signal(key);
                },
                // TODO other types of events
                _ => {}
            }
        }
    }

    pub fn render_root<B, O>(
        &mut self,
        terminal: &mut Terminal<B>,
        root: impl Component<Output = O>,
    ) -> UIResult<O>
    where
        B : Backend,
    {
        let signals = std::mem::take(&mut self.signals);
        let result_holder: Rc<RefCell<Option<UIResult<O>>>> = Rc::new(RefCell::new(None));

        terminal.draw(|frame| {
            let frame_area = frame.area();
            let mut canvas = FrameContext::new(
                frame,
                self,
                frame_area,
            );

            let result = canvas.render_component(RenderArgs::new(&root)
                .key("root")
                .retain_unmounted_state(true)
                .signals(SignalHandling::Overwrite(signals))
            );
            result_holder.borrow_mut().replace(result);
        }).map_err(|err| UIError::IO(err))?;

        self.signals.clear();

        result_holder.take().unwrap()
    }

    pub fn assign_default_attributes(&mut self) {
        self.set_attr(Text::ATTR_COLOR_FG, Color::White);
        self.set_attr(ATTR_COLOR_HIGHLIGHT, Color::Blue);

        self.set_attr(ATTR_KEY_NAV_DOWN, KeyMatcher::new(KeyCode::Down));
        self.set_attr(ATTR_KEY_NAV_UP, KeyMatcher::new(KeyCode::Up));
        self.set_attr(ATTR_KEY_NAV_LEFT, KeyMatcher::new(KeyCode::Left));
        self.set_attr(ATTR_KEY_NAV_RIGHT, KeyMatcher::new(KeyCode::Right));
    }

    pub fn set_attr<T>(&mut self, key: &str, value: T) where T : Any + 'static {
        self.attributes.insert(key.to_string(), Box::new(value));
    }

    pub fn get_attr<T>(&self, key: &str) -> Option<&T> where T : Any + 'static {
        self.attributes.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    pub fn req_attr<T>(&self, attr: &str) -> UIResult<&T> where T : Any + 'static {
        self.attributes.get(attr).and_then(|v| v.downcast_ref::<T>())
            .ok_or(UIError::MissingAttr { attr: attr.to_string() })
    }

    pub fn take_state<T>(&mut self, key: &str) -> T where T : Any + Default + 'static {
        if !self.states.get(key).map(|v| v.is::<T>()).unwrap_or(false) {
            T::default()
        } else {
            *self.states.remove(key).unwrap().downcast::<T>().unwrap()
        }
    }

    pub fn return_state<T>(&mut self, key: &str, state: T) where T : Any + Default + 'static {
        self.states.insert(key.to_string(), Box::new(state));
    }
}