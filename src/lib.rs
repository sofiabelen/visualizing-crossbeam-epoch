#[cfg(not(loom))]
pub use std::sync::atomic::{AtomicPtr, Ordering};
#[cfg(not(loom))]
pub use std::sync::Arc;
#[cfg(not(loom))]
pub use std::thread;

#[cfg(not(loom))]
#[derive(Debug)]
pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

#[cfg(not(loom))]
impl<T> UnsafeCell<T> {
    pub(crate) fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(std::cell::UnsafeCell::new(data))
    }

    pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

#[cfg(loom)]
pub use loom::sync::atomic::{AtomicPtr, Ordering};
#[cfg(loom)]
pub use loom::sync::Arc;
#[cfg(loom)]
pub use loom::thread;
#[cfg(loom)]
pub use loom::cell::UnsafeCell;

pub mod naive_lock_free_stack;

#[cfg(not(loom))]
pub mod aba_problem;