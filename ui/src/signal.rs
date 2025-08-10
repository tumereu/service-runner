use std::any::Any;
use std::rc::Rc;

pub struct Signal(Box<dyn Any + 'static>);
impl Signal {
    pub fn of<T>(payload: T) -> Self
    where
        T: Any + 'static,
    {
        Self(Box::new(payload))
    }

    pub fn is<T>(&self) -> bool
    where
        T: Any + 'static,
    {
        self.0.is::<T>()
    }

    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any + 'static,
    {
        self.0.downcast_ref::<T>()
    }
}

#[derive(Clone, Default)]
pub struct Signals(Vec<Rc<Signal>>);

impl Signals {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn of(vec: Vec<Box<dyn Any + 'static>>) -> Self {
        Self(vec.into_iter().map(|signal| Rc::new(Signal(signal))).collect())
    }

    pub fn singleton<T>(payload: T) -> Self
    where
        T: Any + 'static,
    {
        Self::of(vec![Box::new(payload)])
    }

    pub fn push(&mut self, signal: Signal) {
        self.0.push(Rc::new(signal));
    }

    pub fn merged(left: &Signals, right: &Signals) -> Self {
        Self(
            left.0.iter()
                .chain(right.0.iter())
                .map(|signal| signal.clone())
                .collect()
        )
    }

    pub fn recv<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        for signal in &self.0 {
            if signal.is::<T>() {
                return signal.downcast_ref::<T>().cloned();
            }
        }

        None
    }
}
