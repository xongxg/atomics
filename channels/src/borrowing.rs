use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T: Send> Sync for Channel<T> {}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Channel<T> {
    pub const fn new() -> Channel<T> {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    ///
    /// elide version like following
    /// pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
    pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        *self = Self::new();
        (Sender { channel: self }, Receiver { channel: self })
    }
}

impl<'a, T> Sender<'a, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };

        self.channel.ready.store(true, Release);
    }
}

impl<'a, T> Receiver<'a, T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    pub fn recv(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("no message available!");
        }

        unsafe { (*self.channel.message.get()).assume_init_read() }
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
    fn test() {
        let mut channel = Channel::new();
        thread::scope(|s| {
            let (tx, rx) = channel.split();
            let t = thread::current();
            s.spawn(move || {
                tx.send("hello world!");
                t.unpark();
            });

            while !rx.is_ready() {
                thread::park();
            }

            assert_eq!(rx.recv(), "hello world!");
        });
    }
}
