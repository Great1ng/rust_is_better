use std::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release}
};
    
use std::hint::spin_loop;

/// Test Test-And-Set SpinLock
pub struct TTasSpinLock {
    state: AtomicU32,
}

impl TTasSpinLock {

    #[inline]
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn lock(&self) {
        while self.state.swap(1, Acquire) == 1 {
            while self.state.load(Relaxed) == 1 {
                spin_loop();
            }
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.state.compare_exchange(0, 
                                        1, 
                                        Acquire, 
                                        Relaxed).is_ok()
    }

    #[inline]
    pub fn unlock(&self) {
        self.state.store(0, Release);
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn lock_unlock() {
        let mutex = TTasSpinLock::new();
        mutex.lock();
        mutex.unlock();
    }

    #[test]
    fn try_lock() {
        let mutex = TTasSpinLock::new();
        assert_eq!(mutex.try_lock(), true);
        mutex.unlock();
        mutex.lock();
        assert_eq!(mutex.try_lock(), false);
        mutex.unlock();
    }

}