use crate::mutex_v3::MutexGuard;
use atomic_wait::{wait, wake_all, wake_one};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

pub struct Condvar {
    counter: AtomicU32,
}

impl Condvar {
    pub fn new() -> Condvar {
        Self {
            counter: AtomicU32::new(0),
        }
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(&self) {
        self.counter.fetch_sub(1, Relaxed);
        wake_all(&self.counter);
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let cond_var = self.counter.load(Relaxed);
        let mutex = guard.mutex;
        drop(guard);

        wait(&self.counter, cond_var);
        mutex.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutex_v3::Mutex;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn test_mutex() {
        let mutex = Mutex::new(0);
        let mut wakeups = 0;
        let condvar = Condvar::new();

        thread::scope(|s| {
            s.spawn(|| {
                thread::sleep(Duration::from_secs(1));
                *mutex.lock() = 123;
                condvar.notify_one();
            });

            let mut m = mutex.lock();
            while *m < 100 {
                wakeups += 1;
                m = condvar.wait(m);
            }

            assert_eq!(*m, 123);
        });

        assert!(wakeups < 10);
    }
}
