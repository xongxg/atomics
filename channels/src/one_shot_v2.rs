use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
    in_use: AtomicBool,
}

unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
            in_use: AtomicBool::new(false),
        }
    }

    /// Safety: Only call this once!
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Relaxed) {
            panic!("can't send more than one message!");
        }

        unsafe {
            (*self.message.get()).write(message);
        }
        self.ready.store(true, Release)
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Acquire)
    }

    pub fn receive(&self) -> T {
        if !self.ready.load(Acquire) {
            panic!("no message available!");
        }

        unsafe { (*self.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    #[test]
    fn test_channel() {
        let channel = Channel::new();
        let t = thread::current();

        thread::scope(|s| {
            s.spawn(|| {
                channel.send("hello world!");
                t.unpark();
            });

            while !channel.is_ready() {
                thread::park();
            }

            assert_eq!(channel.receive(), "hello world!");
        });
    }
}
