use std::cell::UnsafeCell;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::sync::Arc;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        message: UnsafeCell::new(MaybeUninit::uninit()),
        ready: AtomicBool::new(false),
    });

    (
        Sender {
            channel: Arc::clone(&channel),
        },
        Receiver { channel },
    )
}

unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Sender<T> {
    pub fn send(self, t: T) {
        unsafe {
            (*self.channel.message.get()).write(t);
        }

        self.channel.ready.store(true, Release);
    }
}

impl<T> Receiver<T> {
    pub fn recv(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("no message available!");
        }

        unsafe { (*self.channel.message.get()).assume_init_read() }
    }

    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe {
                self.message.get_mut().assume_init_drop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_channel() {
        let (tx, rx) = channel();
        let t = thread::current();
        thread::scope(|s| {
            s.spawn(move || {
                tx.send("Hello, World!");
                t.unpark();
            });

            while !rx.is_ready() {
                thread::park();
            }

            assert_eq!(rx.recv(), "Hello, World!");
        });
    }
}
