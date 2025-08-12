use std::any::Any;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Signals(Vec<Rc<dyn Any + 'static>>);

impl Signals {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn of(vec: Vec<Rc<dyn Any + 'static>>) -> Self {
        Self(vec)
    }

    pub fn singleton<T>(payload: T) -> Self
    where
        T: Any + 'static,
    {
        Self::of(vec![Rc::new(payload)])
    }

    pub fn push<T : Any + 'static>(&mut self, signal: T) {
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

    pub fn matching<T>(&self) -> Vec<Rc<T>>
    where
        T: Any + 'static,
    {
        self.0
            .iter().filter(|signal| signal.is::<T>())
            .flat_map(|signal| signal.clone().downcast::<T>().ok())
            .collect()
    }
}
