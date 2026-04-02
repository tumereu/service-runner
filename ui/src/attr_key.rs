use std::any::Any;
use std::marker::PhantomData;

/// A typed attribute key for the renderer's attribute map.
///
/// The type parameter `T` encodes the expected value type for this key,
/// making it a compile-time error to read or write with a mismatched type.
pub struct AttrKey<T: Any + 'static> {
    pub key: &'static str,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: Any + 'static> AttrKey<T> {
    pub const fn new(key: &'static str) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }
}

impl<T: Any + 'static> Copy for AttrKey<T> {}
impl<T: Any + 'static> Clone for AttrKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}
