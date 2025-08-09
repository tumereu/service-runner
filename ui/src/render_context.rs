use std::marker::PhantomData;
use std::rc::Rc;
use crate::state_store::StateStore;

pub struct RenderContext<T>
where
    T: Default + 'static,
{
    store: Rc<StateStore>,
    key: String,
    _marker: PhantomData<T>,
}

impl<T> RenderContext<T>
where
    T: Default + 'static,
{
    pub fn new(store: Rc<StateStore>, key: String) -> Self {
        Self { store, key, _marker: PhantomData }
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