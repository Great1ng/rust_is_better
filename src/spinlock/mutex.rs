use std::{
    cell::UnsafeCell, 
    ops::{Deref, DerefMut}
};
use super::ttas_spinlock::TTasSpinLock;

pub struct Mutex<T> {
    inner: TTasSpinLock,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {

    #[inline]
    pub const fn new(data: T) -> Self {
        Self { 
            inner: TTasSpinLock::new(), 
            data: UnsafeCell::new(data) 
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock();
        MutexGuard::new(self)
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.inner.try_lock().then_some(MutexGuard::new(self))      
    }

    pub fn unlock(guard: MutexGuard<'_, T>) {
        drop(guard);
    }
}

unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<'a, T> MutexGuard<'a, T> {
    fn new(lock: &'a Mutex<T>) -> Self {
        Self { lock }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn shared_counter() {
        let shared_counter = Arc::new(Mutex::new(0u32));
        let iterations = 1000u32;

        let counter_clone = shared_counter.clone();
        let contention = move || {
            for _ in 0..iterations {
                let mut locked_counter = counter_clone.lock();
                *locked_counter += 1;
            }
        };

        thread::spawn(contention.clone()).join().unwrap();
        thread::spawn(contention).join().unwrap();

        assert_eq!(*shared_counter.lock(), 2 * iterations);
    }

}