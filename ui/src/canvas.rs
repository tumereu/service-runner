use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Offset, Rect};
use ratatui::widgets::Widget;
use crate::component::{Component, Measurement};
use crate::space::{Position, Size};
use crate::state_store::{StoreAccessContext, StateStore};

pub struct Canvas<'a, 'b> {
    frame: RefCell<&'a mut Frame<'b>>,
    store: Rc<StateStore>,
    context: RefCell<RenderContext>,
}
impl<'a, 'b> Canvas<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        store: Rc<StateStore>,
        initial_size: Size,
    ) -> Self {
        Self {
            frame: RefCell::new(frame),
            store,
            context: RefCell::new(RenderContext {
                key: String::new(),
                pos: (0i32, 0i32).into(),
                size: initial_size,
            })
        }
    }

    pub fn render_component<S, C>(
        &self,
        args: RenderArgs<S, C>,
    ) where S : Default + 'static, C : Component<S> {
        let RenderArgs {
            key,
            component,
            pos,
            size,
            retain_unmounted_state,
            ..
        } = args;

        let resolved_key = self.resolve_key::<S>(&key);
        let new_context = RenderContext {
            key: resolved_key.clone(),
            pos: &self.context.borrow().pos + pos,
            size,
        };

        let old_context = self.context.replace(new_context);
        self.store.set_retain(&resolved_key, retain_unmounted_state);

        component.render(&self, StoreAccessContext::<S>::new(
            self.store.clone(),
            resolved_key
        ));

        self.context.replace(old_context);
    }

    pub fn render_widget<W>(&self, widget: W, rect: Rect) where W : Widget {
        let pos = &self.context.borrow().pos;
        self.frame.borrow_mut().render_widget(
            widget,
            rect.offset(pos.into())
        );
    }

    pub fn measure_component<S, C>(
        &mut self,
        key: String,
        component: C,
    ) -> Measurement where S : Default + 'static, C : Component<S> {
        let resolved_key = self.resolve_key::<S>(&key);
        let new_context = RenderContext {
            key: resolved_key.clone(),
            pos: self.context.borrow().pos.clone(),
            size: self.context.borrow().size.clone(),
        };

        let old_context = self.context.replace(new_context);

        let measurement = component.measure(&self, StoreAccessContext::<S>::new(
            self.store.clone(),
            resolved_key
        ));

        self.context.replace(old_context);

        measurement
    }

    fn resolve_key<S>(
        &self,
        key: &str,
    ) -> String where S : Default + 'static {
        let typename = std::any::type_name::<S>();
        let current = &self.context.borrow().key;
        // Double the occurrences of '[' to avoid conflicts with the typename included in the resolved key
        let key = key.replace("[", "[[");

        if current.is_empty() {
            format!("{key}[{typename}]")
        } else {
            format!("{current}.{key}[{typename}]")
        }
    }

    pub fn size(&self) -> Size {
        self.context.borrow().size.clone()
    }
}

pub struct RenderContext {
    key: String,
    pos: Position,
    size: Size,
}

pub struct RenderArgs<S, C> where S : Default + 'static, C : Component<S>
{
    pub key: String,
    pub component: C,
    pub pos: Position,
    pub size: Size,
    pub retain_unmounted_state: bool,
    pub state_type: PhantomData<S>,
}