use std::any::Any;
use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::canvas::{FrameContext, RenderArgs};
use crate::component::Component;
use crate::{Signal, SignalHandling, Signals};
use crate::state_store::StateTreeNode;

pub struct RatatuiRenderer {
    store: Rc<StateTreeNode>,
    signals: Signals,
}
impl RatatuiRenderer {
    pub fn new() -> Self {
        Self {
            store: Rc::new(StateTreeNode::new()),
            signals: Signals::empty(),
        }
    }

    pub fn send_signal<T>(&mut self, payload:  T) where T : Any + 'static {
        self.signals.push(Signal::of(payload));
    }

    pub fn render_root<S, B>(
        &mut self,
        terminal: &mut Terminal<B>,
        root: impl Component<State = S>,
    ) -> std::io::Result<()>
    where
        S: Default + 'static,
        B : Backend,
    {
        let signals = std::mem::take(&mut self.signals);

        terminal.draw(|frame| {
            let frame_area = frame.area();
            let canvas = FrameContext::new(
                frame,
                self.store.clone(),
                frame_area
            );

            canvas.render_component(&RenderArgs::new(root)
                .key("root")
                .retain_unmounted_state(true)
                .signals(SignalHandling::Overwrite(signals))
            );
        })?;

        Ok(())
    }
}
