use std::cell::RefCell;
use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Offset, Rect};
use ratatui::widgets::Widget;
use crate::component::{Component, Measurement};
use crate::RenderContext;
use crate::space::{Position, Size};
use crate::state_store::StateTreeNode;

pub struct Canvas<'a, 'b> {
    frame: RefCell<&'a mut Frame<'b>>,
    context: RefCell<Option<ComponentContext>>,
}
impl<'a, 'b> Canvas<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        store: Rc<StateTreeNode>,
        initial_rect: Rect,
    ) -> Self {
        Self {
            frame: RefCell::new(frame),
            context: RefCell::new(Some(ComponentContext {
                area: initial_rect,
                state_node: store,
            }))
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

        let ComponentContext {
            area: current_area,
            state_node: current_state_node,
        } = self.context.take().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        let child_area = Rect {
            x: (current_area.x as i32 + pos.x).try_into().unwrap_or(0),
            y: (current_area.y as i32 + pos.y).try_into().unwrap_or(0),
            width: size.width,
            height: size.height,
        }.intersection(current_area);
        let child_state_node = current_state_node.child(&key, Some(retain_unmounted_state));
        let mut state = child_state_node.take_state::<State>();

        self.context.replace(Some(ComponentContext {
            area: child_area,
            state_node: child_state_node,
        }));
        let output = component.render(&self, &mut state);

        let used_child_context = self.context.replace(Some(ComponentContext {
            area: current_area,
            state_node: current_state_node,
        }));
        used_child_context.unwrap().state_node.return_state(state);

        output
    }

    pub fn render_widget<W>(&self, widget: W, rect: Rect) where W : Widget {
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().expect("Context does not exist -- this indicates a bug in Canvas implementation");
        
        self.frame.borrow_mut().render_widget(
            widget,
            rect.offset(
                Offset {
                    x: ctx.area.x as i32,
                    y: ctx.area.y as i32,
                }
            ).intersection(ctx.area)
        );
    }

    pub fn measure_component<State, C>(
        &mut self,
        key: String,
        component: C,
    ) -> Measurement where State: Default + 'static, C : Component<State = State> {
        let ComponentContext {
            area: current_area,
            state_node: current_state_node,
        } = self.context.take().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        let child_state_node = current_state_node.child(&key, None);
        let mut state = child_state_node.take_state::<State>();

        self.context.replace(Some(ComponentContext {
            area: current_area,
            state_node: child_state_node,
        }));
        let measurement = component.measure(&self, &mut state);

        let used_child_context = self.context.replace(Some(ComponentContext {
            area: current_area,
            state_node: current_state_node,
        }));
        used_child_context.unwrap().state_node.return_state(state);

        measurement
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.context.borrow().area.width,
            height: self.context.borrow().area.height,
        }
    }
}

pub struct ComponentContext {
    area: Rect,
    state_node: Rc<StateTreeNode>,
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
