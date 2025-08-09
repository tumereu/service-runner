use std::cell::RefCell;
use std::rc::Rc;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::canvas::Canvas;
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

    pub fn render_root<S, B>(&self, terminal: &mut Terminal<B>, root: impl Component<S>) -> std::io::Result<()>
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

            canvas.render_component(
                "root".to_owned(),
                root,
                (0, 0).into(),
                (frame_size.width, frame_size.height).into(),
                Default::default(),
            );
        })?;

        Ok(())
    }
}
