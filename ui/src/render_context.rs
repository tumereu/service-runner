use std::marker::PhantomData;
use std::rc::Rc;
use crate::state_store::StateTreeNode;

pub struct RenderContext<T>
where
    T: Default + 'static,
{
    state_store: Rc<StateTreeNode>,
    key: String,
    _marker: PhantomData<T>,
}

impl<T> RenderContext<T>
where
    T: Default + 'static,
{
    pub fn new(store: Rc<StateTreeNode>, key: String) -> Self {
        Self { state_store: store, key, _marker: PhantomData }
    }
}