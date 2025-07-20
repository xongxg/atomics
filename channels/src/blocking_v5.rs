use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::thread::Thread;

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Channel<T> {
    pub const fn new() -> Channel<T> {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        *self = Self::new();
        (
            Sender {
                channel: self,
                receiving_thread: thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread,
}

impl<'a, T> Sender<'a, T> {
    pub fn send(self, value: T) {
        unsafe {
            (*self.channel.message.get()).write(value);
        }

        self.receiving_thread.unpark();
        self.channel.ready.store(true, Release);
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const ()>,
}
impl<'a, T> Receiver<'a, T> {
    pub fn recv(self) -> T {
        while !self.channel.ready.swap(false, Release) {
            thread::park();
        }

        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<'a, T> Drop for Receiver<'a, T> {
    fn drop(&mut self) {
        if self.channel.ready.load(Acquire) {
            unsafe { (*self.channel.message.get()).assume_init_drop() }
        }
    }
}


#[test]
fn main() {
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("hello world!");
        });
        assert_eq!(receiver.recv(), "hello world!");
    });
}