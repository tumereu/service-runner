use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct StateTreeNode {
    state: RefCell<Option<Box<dyn Any + 'static>>>,
    children: RefCell<BTreeMap<String, Rc<StateTreeNode>>>,
}
impl StateTreeNode {
    pub fn new() -> Self {
        Self {
            state: RefCell::new(None),
            children: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn take_state<T>(&self) -> Box<T> where T: Default + 'static {
        if self.state.borrow().as_ref().map(|value| !value.is::<T>()).unwrap_or(true) {
            self.state.replace(Some(Box::new(T::default())));
        }

        self.state.borrow_mut().take().unwrap().downcast::<T>().unwrap()
    }

    pub fn return_state<T>(&self, state: Box<T>) where T: Default + 'static {
        self.state.borrow_mut().replace(state);
    }

    pub fn child(&self, key: &str, retain: Option<bool>) -> Rc<Self> {
        let mut children = self.children.borrow_mut();

        if !children.contains_key(key) {
            children.insert(key.to_string(), Rc::new(StateTreeNode::new()));
        }

        children.get(key).unwrap().clone()
    }
}

