use std::cell::UnsafeCell;
use std::io::Empty;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// state for channel
const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const READING: u8 = 3;

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

impl<T> Channel<T> {
    pub fn new(message: T) -> Channel<T> {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(EMPTY),
        }
    }

    pub fn send(&self, value: T) {
        if self
            .state
            .compare_exchange_weak(EMPTY, WRITING, Relaxed, Relaxed)
            .is_err()
        {
            panic!("can't send more than one message!");
        }

        unsafe {
            (*self.message.get()).write(value);
        }

        self.state.store(READY, Release)
    }

    pub fn is_ready(&self) -> bool {
        self.state.load(Relaxed) == READY
    }

    pub fn recv(&self) -> T {
        if self
            .state
            .compare_exchange_weak(READY, READING, Acquire, Relaxed)
            .is_err()
        {
            panic!("can't receive more than one message!");
        }

        unsafe { (*self.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == READY {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}
