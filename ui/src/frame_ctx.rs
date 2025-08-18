use crate::component::{Component, MeasurableComponent};
use crate::signals::Signals;
use crate::space::Position;
use crate::{ComponentRenderer, UIError, UIResult};
use ratatui::layout::{Rect, Size};
use ratatui::widgets::Widget;
use ratatui::Frame;
use std::any::Any;

pub struct FrameContext<'a, 'b, 'c> {
    frame: &'a mut Frame<'b>,
    current_area: Rect,
    signals: Signals,
    renderer: &'c mut ComponentRenderer
}
impl<'a, 'b, 'c> FrameContext<'a, 'b, 'c> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        renderer: &'c mut ComponentRenderer,
        initial_rect: Rect,
    ) -> Self {
        Self {
            frame,
            current_area: initial_rect,
            signals: Signals::empty(),
            renderer,
        }
    }

    pub fn render_component<Output, C>(
        &mut self,
        args: RenderArgs<Output, C>,
    ) -> Result<Output, UIError> where C : Component<Output = Output> {
        let RenderArgs {
            component,
            pos,
            size,
            signals: signal_handling,
        } = args;

        let pos = pos.as_ref().cloned().unwrap_or_default();
        let size = size.as_ref().cloned().unwrap_or(self.size());

        let new_area = Rect {
            x: (self.current_area.x as i32 + pos.x).try_into().unwrap_or(0),
            y: (self.current_area.y as i32 + pos.y).try_into().unwrap_or(0),
            width: size.width,
            height: size.height,
        }.intersection(self.current_area);
        
        let old_signals = match signal_handling {
            SignalHandling::Forward => None,
            SignalHandling::Overwrite(signals) => Some(self.signals.overwrite(signals)),
            SignalHandling::Add(signals) => Some(self.signals.extend(signals)),
            SignalHandling::Block => Some(self.signals.take()),
        };
        let old_area = std::mem::replace(&mut self.current_area, new_area);

        let output = component.render(self)
            .map_err(|err| err.nested::<C>());

        self.current_area = old_area;
        if let Some(signals) = old_signals {
            self.signals = signals;
        }

        output
    }

    pub fn render_widget<W>(&mut self, widget: W, pos: Position, size: Size) where W : Widget {
        self.frame.render_widget(
            widget,
            Rect {
                x: (self.current_area.x as i32 + pos.x).try_into().unwrap_or(0),
                y: (self.current_area.y as i32 + pos.y).try_into().unwrap_or(0),
                width: size.width,
                height: size.height,
            }.intersection(self.current_area),
        );
    }

    pub fn measure_component<C>(
        &self,
        component: &C,
    ) -> UIResult<Size> where C : MeasurableComponent {
        let measurement = component.measure(&self)
            .map_err(|err| err.nested::<C>());

        measurement
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.current_area.width,
            height: self.current_area.height,
        }
    }

    pub fn area(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: self.current_area.width,
            height: self.current_area.height,
        }
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    pub fn get_attr<T>(&self, key: &str) -> Option<&T> where T : Any + 'static {
        self.renderer.get_attr(key)
    }

    pub fn req_attr<T>(&self, attr: &str) -> UIResult<&T> where T : Any + 'static {
        self.renderer.req_attr(attr)
    }

    pub fn take_state<T>(&mut self, key: &str) -> T where T : Any + Default + 'static {
        self.renderer.take_state(key)
    }

    pub fn return_state<T>(&mut self, key: &str, state: T) where T : Any + Default + 'static {
        self.renderer.return_state(key, state);
    }
}

#[derive(Clone)]
pub struct RenderArgs<Output, C> where C : Component<Output = Output>
{
    pub component: C,
    pub pos: Option<Position>,
    pub size: Option<Size>,
    pub signals: SignalHandling,
}
impl<'a, Output, C> RenderArgs<Output, C> where C : Component<Output = Output> {
    pub fn new(component: C) -> RenderArgs<Output, C> {
        RenderArgs {
            component,
            pos: None,
            size: None,
            signals: SignalHandling::Forward,
        }
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