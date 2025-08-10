use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::canvas::{Canvas, RenderArgs};
use crate::component::Component;
use crate::state_store::StateTreeNode;

pub struct RatatuiRenderer {
    store: Rc<StateTreeNode>,
}
impl RatatuiRenderer {
    pub fn new() -> Self {
        Self {
            store: Rc::new(StateTreeNode::new())
        }
    }

    pub fn render_root<S, B>(&self, terminal: &mut Terminal<B>, root: impl Component<State = S>) -> std::io::Result<()>
    where
        S: Default + 'static,
        B : Backend,
    {
        terminal.draw(|frame| {
            let frame_area = frame.area();
            let canvas = Canvas::new(
                frame,
                self.store.clone(),
                frame_area
            );

            canvas.render_component(RenderArgs {
                key: "root".to_string(),
                component: root,
                pos: (0, 0).into(),
                size: (frame_area.width, frame_area.height).into(),
                retain_unmounted_state: true,
            });
        })?;

        Ok(())
    }
}
