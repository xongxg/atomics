use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, mpsc};

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, item: T) {
        self.queue.lock().unwrap().push_back(item);
        self.item_ready.notify_one()
    }

    pub fn recv(&self) -> T {
        let mut b = self.queue.lock().unwrap();

        loop {
            if let Some(x) = b.pop_front() {
                return x;
            }

            b = self.item_ready.wait(b).unwrap();
        }
    }
}
