use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
pub struct StateStore {
    states: RefCell<BTreeMap<String, Box<dyn Any + 'static>>>,
}
impl StateStore {
    pub fn new() -> Self {
        Self {
            states: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn access<R, F, T>(&self, key: &str, block: F) -> R
    where
        R: 'static,
        T: Default + 'static,
        for<'a> F: FnOnce(&'a mut T) -> R,
    {
        let mut states = self.states.borrow_mut();
        if !states.contains_key(key) {
            states.insert(key.to_string(), Box::new(T::default()));
        }

        states
            .get_mut(key)
            .and_then(|value| value.downcast_mut::<T>())
            .map(|value| block(value))
            .expect("Error in access(): expected T to exist, but it did not. Downcast failure?")
    }

    pub fn set_retain(&self, key: &str, retain: bool) {
        // TODO: Implement this
    }
}

