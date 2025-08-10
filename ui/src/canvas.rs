use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Offset, Rect};
use ratatui::widgets::Widget;
use log::debug;
use crate::component::{Component, Measurement};
use crate::RenderContext;
use crate::signal::Signals;
use crate::space::{Position, Size};
use crate::state_store::StateTreeNode;

pub struct FrameContext<'a, 'b> {
    frame: RefCell<&'a mut Frame<'b>>,
    current: RefCell<Option<CurrentComponentContext>>,
}
impl<'a, 'b> FrameContext<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        store: Rc<StateTreeNode>,
        initial_rect: Rect,
    ) -> Self {
        Self {
            frame: RefCell::new(frame),
            current: RefCell::new(Some(CurrentComponentContext {
                area: initial_rect,
                state_node: store,
                signals: Signals::empty(),
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
            signals: signal_handling,
        } = args;

        let CurrentComponentContext {
            area: current_area,
            state_node: current_state_node,
            signals: current_signals,
        } = self.current.take().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        let child_area = Rect {
            x: (current_area.x as i32 + pos.x).try_into().unwrap_or(0),
            y: (current_area.y as i32 + pos.y).try_into().unwrap_or(0),
            width: size.width,
            height: size.height,
        }.intersection(current_area);
        let child_state_node = current_state_node.child(&key, Some(retain_unmounted_state));
        let mut state = child_state_node.take_state::<State>();

        self.current.replace(Some(CurrentComponentContext {
            area: child_area,
            state_node: child_state_node,
            signals: match signal_handling {
                SignalHandling::Overwrite(new_signals) => new_signals,
                SignalHandling::Block => Signals::empty(),
                SignalHandling::Forward => current_signals.clone(),
                SignalHandling::Add(added_signals) => Signals::merged(&current_signals, &added_signals),
            }
        }));
        let output = component.render(&self, &mut state);

        let used_child_context = self.current.replace(Some(CurrentComponentContext {
            area: current_area,
            state_node: current_state_node,
            signals: current_signals,
        }));
        used_child_context.unwrap().state_node.return_state(state);

        output
    }

    pub fn render_widget<W>(&self, widget: W, rect: Rect) where W : Widget {
        let ctx = self.current.borrow();
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
        let CurrentComponentContext {
            area: current_area,
            state_node: current_state_node,
            signals,
        } = self.current.take().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        let child_state_node = current_state_node.child(&key, None);
        let state = child_state_node.take_state::<State>();

        self.current.replace(Some(CurrentComponentContext {
            area: current_area,
            state_node: child_state_node,
            signals: Signals::empty(),
        }));
        let measurement = component.measure(&self, &state);

        let used_child_context = self.current.replace(Some(CurrentComponentContext {
            area: current_area,
            state_node: current_state_node,
            signals
        }));
        used_child_context.unwrap().state_node.return_state(state);

        measurement
    }

    pub fn size(&self) -> Size {
        let ctx = self.current.borrow();
        let ctx = ctx.as_ref().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        Size {
            width: ctx.area.width,
            height: ctx.area.height,
        }
    }

    pub fn on_signal<T, R>(&self, handle: impl FnOnce(T) -> R) -> Option<R>
    where
        T: Clone + 'static,
        R: 'static,
    {
        let ctx = self.current.borrow();
        let ctx = ctx.as_ref().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        ctx.signals.recv::<T>().map(handle)
    }
}

pub struct CurrentComponentContext {
    area: Rect,
    state_node: Rc<StateTreeNode>,
    signals: Signals,
}

pub struct RenderArgs<State, Output, C> where State: Default + 'static, C : Component<State = State, Output = Output>
{
    pub key: String,
    pub component: C,
    pub pos: Position,
    pub size: Size,
    pub signals: SignalHandling,
    pub retain_unmounted_state: bool,
}

pub enum SignalHandling {
    Overwrite(Signals),
    Add(Signals),
    Forward,
    Block
}
impl Default for SignalHandling {
    fn default() -> Self {
        Self::Forward
    }
}

#[macro_export]
macro_rules! render {
    // Entry point: capture the canvas variable and a block of key=value pairs
    ($canvas:expr, {
        key = $key:expr,
        component = $component:expr,
        pos = $pos:expr,
        $( size = $size:expr, )?
        $( signals = $signals:expr, )?
        $( retain_unmounted_state = $retain:expr, )?
    }) => {{
        // Build RenderArgs inline
        let args = $crate::RenderArgs {
            key: $key.to_string(),
            pos: $pos.into(),
            component: $component,
            size: render!($($size)?).unwrap_or_else(|| $canvas.size()),
            signals: render!($($signals)?).unwrap_or_default(),
            retain_unmounted_state: render!($($retain)?).unwrap_or(false),
        };
        $canvas.render_component(args);
    }};

    () => { None };
    ($entity:expr) => { Some($entity) }
}
