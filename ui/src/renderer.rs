use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::style::Color;
use ratatui::Terminal;
use crate::frame_ctx::{FrameContext, RenderArgs};
use crate::component::{Component, Text};
use crate::{UIError, Signal, SignalHandling, Signals, UIResult};
use crate::state_store::StateTreeNode;

pub struct ComponentRenderer {
    store: Rc<StateTreeNode>,
    signals: Signals,
    attributes: HashMap<String, Box<dyn Any>>,
}
impl ComponentRenderer {
    pub fn new() -> Self {
        Self {
            store: Rc::new(StateTreeNode::new()),
            signals: Signals::empty(),
            attributes: HashMap::new(),       
        }
    }

    pub fn send_signal<T>(&mut self, payload:  T) where T : Any + 'static {
        self.signals.push(Signal::of(payload));
    }

    pub fn render_root<S, B, O>(
        &mut self,
        terminal: &mut Terminal<B>,
        root: impl Component<State = S, Output = O>,
    ) -> UIResult<O>
    where
        S: Default + 'static,
        B : Backend,
    {
        let signals = std::mem::take(&mut self.signals);
        let result_holder: Rc<RefCell<Option<UIResult<O>>>> = Rc::new(RefCell::new(None));

        terminal.draw(|frame| {
            let frame_area = frame.area();
            let canvas = FrameContext::new(
                frame,
                self.store.clone(),
                &self,
                frame_area,
            );

            let result = canvas.render_component(RenderArgs::new(&root)
                .key("root")
                .retain_unmounted_state(true)
                .signals(SignalHandling::Overwrite(signals))
            );
            result_holder.borrow_mut().replace(result);
        }).map_err(|err| UIError::IO(err))?;

        result_holder.take().unwrap()
    }
    
    pub fn assign_default_attributes(&mut self) {
        self.set_attr(Text::ATTR_DEFAULT_FG, Color::White);
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
}