use std::cell::RefCell;
use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Offset, Rect};
use ratatui::widgets::Widget;
use crate::component::{Component, Measurement};
use crate::RenderContext;
use crate::space::{Position, Size};
use crate::state_store::StateStore;

pub struct Canvas<'a, 'b> {
    frame: RefCell<&'a mut Frame<'b>>,
    store: Rc<StateStore>,
    context: RefCell<ComponentContext>,
}
impl<'a, 'b> Canvas<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        store: Rc<StateStore>,
        initial_rect: Rect,
    ) -> Self {
        Self {
            frame: RefCell::new(frame),
            store,
            context: RefCell::new(ComponentContext {
                key: String::new(),
                area: initial_rect
            })
        }
    }

    pub fn render_component<State, Output, C>(
        &self,
        args: RenderArgs<State, Output, C>,
    ) -> Output where State: Default + 'static, C : Component<State = State, Output = Output> {
        let RenderArgs {
            key,
            component,
            pos,
            size,
            retain_unmounted_state,
            ..
        } = args;

        let resolved_key = self.resolve_key::<State>(&key);
        let component_area = Rect {
            x: (self.context.borrow().area.x as i32 + pos.x).try_into().unwrap_or(0),
            y: (self.context.borrow().area.y as i32 + pos.y).try_into().unwrap_or(0),
            width: size.width,
            height: size.height,
        }.intersection(self.context.borrow().area);
        let new_context = ComponentContext {
            key: resolved_key.clone(),
            area: component_area,
        };

        let old_context = self.context.replace(new_context);
        self.store.set_retain(&resolved_key, retain_unmounted_state);

        let output = component.render(&self, RenderContext::<State>::new(
            self.store.clone(),
            resolved_key
        ));

        self.context.replace(old_context);

        output
    }

    pub fn render_widget<W>(&self, widget: W, rect: Rect) where W : Widget {
        self.frame.borrow_mut().render_widget(
            widget,
            rect.offset(
                Offset {
                    x: self.context.borrow().area.x as i32,
                    y: self.context.borrow().area.y as i32,
                }
            ).intersection(self.context.borrow().area)
        );
    }

    pub fn measure_component<State, C>(
        &mut self,
        key: String,
        component: C,
    ) -> Measurement where State: Default + 'static, C : Component<State = State> {
        let resolved_key = self.resolve_key::<State>(&key);
        let new_context = ComponentContext {
            key: resolved_key.clone(),
            area: self.context.borrow().area,
        };

        let old_context = self.context.replace(new_context);

        let measurement = component.measure(&self, RenderContext::<State>::new(
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
        Size {
            width: self.context.borrow().area.width,
            height: self.context.borrow().area.height,
        }
    }
}

pub struct ComponentContext {
    key: String,
    area: Rect,
}

pub struct RenderArgs<State, Output, C> where State: Default + 'static, C : Component<State = State, Output = Output>
{
    pub key: String,
    pub component: C,
    pub pos: Position,
    pub size: Size,
    pub retain_unmounted_state: bool,
}

#[macro_export]
macro_rules! render {
    // Entry point: capture the canvas variable and a block of key=value pairs
    ($canvas:expr, {
        key = $key:expr,
        component = $component:expr,
        pos = $pos:expr,
        $( size = $size:expr, )?
        $( retain_unmounted_state = $retain:expr, )?
    }) => {{
        // Build RenderArgs inline
        let args = $crate::RenderArgs {
            key: $key.to_string(),
            pos: $pos.into(),
            component: $component,
            size: render!($($size)?).unwrap_or_else(|| $canvas.size()),
            retain_unmounted_state: render!($($retain)?).unwrap_or(false),
        };
        $canvas.render_component(args);
    }};

    () => { None };
    ($entity:expr) => { Some($entity) }
}
