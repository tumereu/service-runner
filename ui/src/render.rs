use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::canvas::{Canvas, RenderArgs};
use crate::component::Component;
use crate::state_store::StateStore;

pub struct RatatuiRenderer {
    store: Rc<StateStore>,
}
impl RatatuiRenderer {
    pub fn new() -> Self {
        Self {
            store: Rc::new(StateStore::new())
        }
    }

    pub fn render_root<S, B>(&self, terminal: &mut Terminal<B>, root: impl Component<State = S>) -> std::io::Result<()>
    where
        S: Default + 'static,
        B : Backend,
    {
        terminal.draw(|frame| {
            let frame_size = frame.area();
            let canvas = Canvas::new(
                frame,
                self.store.clone(),
                (frame_size.width, frame_size.height).into(),
            );

            canvas.render_component(RenderArgs {
                key: "root".to_string(),
                component: root,
                pos: (0, 0).into(),
                size: (frame_size.width, frame_size.height).into(),
                retain_unmounted_state: true,
            });
        })?;

        Ok(())
    }
}
