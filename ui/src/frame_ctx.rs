use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Offset, Rect, Size};
use ratatui::widgets::Widget;
use log::debug;
use crate::component::{Component, MeasurableComponent};
use crate::signal::Signals;
use crate::space::{Position};
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

        let size = size.as_ref().cloned().unwrap_or(self.size());
        let pos = pos.as_ref().cloned().unwrap_or_default();

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
        let child_state_node = current_state_node.child(
            key.unwrap(),
            Some(retain_unmounted_state)
        );
        let mut state = child_state_node.take_state::<State>();

        self.current.replace(Some(CurrentComponentContext {
            area: child_area,
            state_node: child_state_node,
            signals: match signal_handling {
                SignalHandling::Overwrite(new_signals) => new_signals.clone(),
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
        &self,
        key: &str,
        component: &C,
    ) -> Size where State: Default + 'static, C : MeasurableComponent<State = State> {
        let CurrentComponentContext {
            area: current_area,
            state_node: current_state_node,
            signals,
        } = self.current.take().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        let child_state_node = current_state_node.child(key, None);
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

    pub fn area(&self) -> Rect {
        let ctx = self.current.borrow();
        let ctx = ctx.as_ref().expect("Context does not exist -- this indicates a bug in Canvas implementation");

        Rect {
            x: 0,
            y: 0,
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

#[derive(Clone)]
pub struct RenderArgs<'a, State, Output, C> where State: Default + 'static, C : Component<State = State, Output = Output>
{
    pub key: Option<&'a str>,
    pub component: &'a C,
    pub pos: Option<Position>,
    pub size: Option<Size>,
    pub signals: SignalHandling,
    pub retain_unmounted_state: bool,
}
impl<'a, State, Output, C> RenderArgs<'a, State, Output, C> where State: Default + 'static, C : Component<State = State, Output = Output> {
    pub fn new(component: &'a C) -> RenderArgs<'a, State, Output, C> {
        RenderArgs {
            key: None,
            component,
            pos: None,
            size: None,
            signals: SignalHandling::Forward,
            retain_unmounted_state: false,
        }
    }

    pub fn from(other: &RenderArgs<'a, State, Output, C>) -> RenderArgs<'a, State, Output, C> {
        RenderArgs {
            key: other.key.clone(),
            component: other.component.clone(),
            pos: other.pos.clone(),
            size: other.size.clone(),
            signals: other.signals.clone(),
            retain_unmounted_state: other.retain_unmounted_state,
        }
    }

    pub fn key(self, key: &'a str) -> Self {
        let mut self_mut = self;
        self_mut.key = Some(key);
        self_mut
    }

    pub fn pos<X : Into<i32>, Y: Into<i32>>(self, x: X, y: Y) -> Self {
        let mut self_mut = self;
        self_mut.pos = Some((x, y).into());
        self_mut
    }

    pub fn size<X : Into<u16>, Y: Into<u16>>(self, width: X, height: Y) -> Self {
        let mut self_mut = self;
        self_mut.size = Some(Size { width: width.into(), height: height.into() });
        self_mut
    }

    pub fn signals(self, signals: SignalHandling) -> Self {
        let mut self_mut = self;
        self_mut.signals = signals;
        self_mut
    }

    pub fn retain_unmounted_state(self, retain_unmounted_state: bool) -> Self {
        let mut self_mut = self;
        self_mut.retain_unmounted_state = retain_unmounted_state;
        self_mut
    }
}

#[derive(Clone)]
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