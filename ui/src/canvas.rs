use std::cell::RefCell;
use std::rc::Rc;
use ratatui::Frame;
use crate::component::{Component, Measurement};
use crate::space::Position;
use crate::store::{StoreAccessContext, Store};

pub struct Canvas<'a> {
    frame: RefCell<&'a mut Frame<'a>>,
    store: Rc<Store>,
    context: RefCell<RenderContext>,
}
impl<'a> Canvas<'a> {
    pub fn render_component<S, C>(
        &self,
        key: String,
        component: C,
        pos: Position,
        args: RenderArgs,
    ) where S : Default + 'static, C : Component<S> {
        let resolved_key = self.resolve_key::<S>(&key);
        let new_context = RenderContext {
            key: resolved_key.clone(),
            pos: &self.context.borrow().pos + pos,
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
        };

        let old_context = self.context.replace(new_context);
        let state = self.store.obtain::<S>(&resolved_key);

        let measurement = component.measure(&self, &state);

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
}

#[derive(Default)]
pub struct RenderArgs {
    pub retain_unmounted_state: bool,
}