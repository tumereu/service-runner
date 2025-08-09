use std::cell::{Ref, RefCell};
use std::rc::Rc;
use ratatui::Frame;
use crate::component::{Component, Measurement};
use crate::space::{Position, Size};
use crate::state_store::{StoreAccessContext, StateStore};

pub struct Canvas<'a, 'b> {
    frame: &'a mut Frame<'b>,
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
            frame,
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
        key: String,
        component: C,
        pos: Position,
        size: Size,
        args: RenderArgs,
    ) where S : Default + 'static, C : Component<S> {
        let resolved_key = self.resolve_key::<S>(&key);
        let new_context = RenderContext {
            key: resolved_key.clone(),
            pos: &self.context.borrow().pos + pos,
            size,
        };

        let old_context = self.context.replace(new_context);
        self.store.set_retain(&resolved_key, args.retain_unmounted_state);

        component.render(&self, StoreAccessContext::<S>::new(
            self.store.clone(),
            resolved_key
        ));

        self.context.replace(old_context);
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
}

pub struct RenderContext {
    key: String,
    pos: Position,
    size: Size,
}

#[derive(Default)]
pub struct RenderArgs {
    pub retain_unmounted_state: bool,
}