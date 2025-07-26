use atomic_wait::{wait, wake_one};
use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::atomic::{AtomicU32, AtomicUsize};

pub struct Mutex<T> {
    value: UnsafeCell<T>,

    /// unlocked: 0
    /// locked : 1
    state: AtomicU32,
}

unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: 'a> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Self {
        Mutex {
            value: UnsafeCell::new(t),
            state: AtomicU32::new(0),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self.state.swap(1, Acquire) == 1 {
            wait(&self.state, 1);
        }

        MutexGuard { mutex: self }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.state.store(0, Release);
        wake_one(&self.mutex.state);
    }
}
