use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::frame_ctx::{FrameContext, RenderArgs};
use crate::component::Component;
use crate::{UIError, Signal, SignalHandling, Signals, UIResult};
use crate::state_store::StateTreeNode;

pub struct ComponentRenderer {
    store: Rc<StateTreeNode>,
    signals: Signals,
}
impl ComponentRenderer {
    pub fn new() -> Self {
        Self {
            store: Rc::new(StateTreeNode::new()),
            signals: Signals::empty(),
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
                frame_area
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
}
