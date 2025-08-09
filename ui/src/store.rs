use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct Store {
    states: RefCell<BTreeMap<String, Box<dyn Any + 'static>>>,
}
impl Store {
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

pub struct StoreAccessContext<T>
where
    T: Default + 'static,
{
    store: Rc<Store>,
    key: String,
}
impl<T> StoreAccessContext<T>
where
    T: Default + 'static,
{
    pub fn new(store: Rc<Store>, key: String) -> Self {
        Self { store, key }
    }

    pub fn query<R, F>(&self, query: F) -> R
    where
        R: 'static,
        for<'a> F: FnOnce(&'a T) -> R,
    {
        self.store.access(&self.key, |state| query(state))
    }

    pub fn update<R, F>(&self, update: F) -> R
    where
        R: 'static,
        for<'a> F: FnOnce(&'a mut T) -> R,
    {
        self.store.access(&self.key, |state| update(state))
    }
}
