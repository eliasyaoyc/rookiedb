use std::fmt::{self, Debug, Pointer};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicUsize, Ordering};

#[derive(Debug)]
struct Value<T> {
    refs: AtomicUsize,
    data: T,
}

impl<T> Value<T> {
    #[inline]
    fn new(data: T) -> Self {
        Value {
            refs: AtomicUsize::new(1),
            data,
        }
    }

    #[inline]
    fn increase(&self) -> usize {
        self.refs.fetch_add(1, Ordering::Release)
    }

    #[inline]
    fn decrease(&self) -> usize {
        self.refs.fetch_sub(1, Ordering::Release) - 1
    }
}

struct ValueWrapper<T> {
    // The pointor to `Value`.
    ptr: AtomicUsize,
    // zero-cost abstration.
    phantom: PhantomData<T>,
}

impl<T> ValueWrapper<T> {
    /// Transmute raw pointor to `Value`.
    /// First we needs remove write-tag-bit and read-tag-bit
    #[inline]
    fn transmute(&self, ptr: usize) -> Option<&Value<T>> {
        let ptr = ptr & !3;
        if ptr == 0 {
            return None;
        }
        Some(unsafe { &*(ptr as *const Value<T>) })
    }

    #[inline]
    fn is_none(&self) -> bool {
        let ptr = self.ptr.load(Ordering::Acquire);
        let ptr = ptr & !3;
        if ptr == 0 {
            return true;
        }
        false
    }

    #[inline]
    fn is_locked(&self) -> bool {
        let ptr = self.ptr.load(Ordering::Acquire);
        ptr & 1 == 1
    }

    #[inline]
    fn get(&self) -> Option<RcuReader<T>> {
        let ptr = self.read_lock();

        let reader = self.transmute(ptr).map(|value| {
            value.increase();
            RcuReader {
                inner: NonNull::new(value as *const Value<T> as *mut Value<T>)
                    .expect("null shared."),
            }
        });
        self.read_unlock();
        reader
    }

    #[inline]
    fn swap(&self, data: Option<T>) -> Option<&Value<T>> {
        let new = if let Some(v) = data {
            Box::into_raw(Box::new(Value::new(v))) as usize | 1
        } else {
            1
        };

        let mut old = self.ptr.load(Ordering::Acquire) & !2;

        loop {
            match self
                .ptr
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(x) => {
                    old = x & !2;
                    core::hint::spin_loop();
                }
            }
        }
        self.transmute(old)
    }

    #[inline]
    fn acquire(&self) -> bool {
        let mut old = self.ptr.load(Ordering::Acquire);
        // Already written thread hold.
        if old & 1 != 0 {
            return false;
        }

        loop {
            let new = old | 1;
            match self
                .ptr
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return true,
                Err(x) if x & 1 == 0 => old = x,
                _ => return false,
            }
        }
    }

    #[inline]
    fn release(&self) {
        self.ptr.fetch_and(!1, Ordering::Release);
    }

    #[inline]
    fn read_lock(&self) -> usize {
        let mut old = self.ptr.load(Ordering::Acquire) & !2;
        loop {
            let new = old | 2;
            match self
                .ptr
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return new,
                Err(x) => {
                    old = x & !2;
                    core::hint::spin_loop();
                }
            }
        }
    }

    #[inline]
    fn read_unlock(&self) {
        self.ptr.fetch_and(!2, Ordering::Release);
    }
}

impl<T> Drop for ValueWrapper<T> {
    fn drop(&mut self) {
        if let Some(reader) = self.get() {
            reader.evict();
        }
    }
}

impl<T: Debug> Debug for ValueWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.ptr.load(Ordering::Acquire);
        let value = self.transmute(ptr);
        f.debug_struct("RCUValue").field("data", &value).finish()
    }
}

pub struct RcuReader<T> {
    inner: NonNull<Value<T>>,
}

unsafe impl<T: Send> Send for RcuReader<T> {}
unsafe impl<T: Sync> Sync for RcuReader<T> {}

impl<T> Drop for RcuReader<T> {
    fn drop(&mut self) {
        unsafe {
            if self.inner.as_ref().decrease() == 0 {
                atomic::fence(Ordering::Acquire);
                let _: Box<Value<T>> = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}

impl<T> Deref for RcuReader<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.inner.as_ref().data }
    }
}

#[allow(clippy::explicit_auto_deref)]
impl<T> AsRef<T> for RcuReader<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> Clone for RcuReader<T> {
    fn clone(&self) -> Self {
        unsafe {
            let cnt = self.inner.as_ref().increase();
            assert!(cnt > 0);
        }
        RcuReader { inner: self.inner }
    }
}

impl<T: PartialEq> PartialEq for RcuReader<T> {
    fn eq(&self, other: &Self) -> bool {
        *(*self) == *(*other)
    }
}

impl<T: PartialOrd> PartialOrd for RcuReader<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }

    fn lt(&self, other: &Self) -> bool {
        *(*self) < *(*other)
    }

    fn le(&self, other: &Self) -> bool {
        *(*self) <= *(*other)
    }

    fn gt(&self, other: &Self) -> bool {
        *(*self) > *(*other)
    }

    fn ge(&self, other: &Self) -> bool {
        *(*self) >= *(*other)
    }
}

impl<T: Ord> Ord for RcuReader<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: Eq> Eq for RcuReader<T> {}

impl<T: Debug> Debug for RcuReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> Pointer for RcuReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

impl<T> RcuReader<T> {
    #[inline]
    fn evict(&self) {
        unsafe {
            self.inner.as_ref().decrease();
        }
    }
}

pub struct RcuGuard<'a, T> {
    wrapper: &'a ValueWrapper<T>,
}

unsafe impl<'a, T: Sync> Sync for RcuGuard<'a, T> {}

impl<'a, T> RcuGuard<'a, T> {
    pub fn update(&mut self, data: Option<T>) {
        let old = self.wrapper.swap(data);
        if let Some(v) = old {
            let cnt = v.increase();
            assert!(cnt > 0);
            let ptr = NonNull::new(v as *const Value<T> as *mut Value<T>).expect("null shared");
            let reader = RcuReader::<T> { inner: ptr };
            reader.evict();
        }
    }

    pub fn as_ref(&self) -> Option<&T> {
        let ptr = self.wrapper.ptr.load(Ordering::Acquire) & !3;
        if ptr == 0 {
            return None;
        }
        let value = unsafe { &*(ptr as *const Value<T>) };
        Some(&value.data)
    }
}

impl<'a, T> Drop for RcuGuard<'a, T> {
    fn drop(&mut self) {
        self.wrapper.release();
    }
}

impl<'a, T: Debug> Debug for RcuGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_ref(), f)
    }
}

#[derive(Debug)]
pub struct RcuCell<T> {
    inner: ValueWrapper<T>,
}

unsafe impl<T: Send> Send for RcuCell<T> {}
unsafe impl<T: Sync> Sync for RcuCell<T> {}

impl<T> Default for RcuCell<T> {
    fn default() -> Self {
        RcuCell::new(None)
    }
}

impl<T> RcuCell<T> {
    pub fn new(data: Option<T>) -> Self {
        let ptr = if let Some(data) = data {
            Box::into_raw(Box::new(Value::new(data))) as usize
        } else {
            0
        };
        RcuCell {
            inner: ValueWrapper {
                ptr: AtomicUsize::new(ptr),
                phantom: PhantomData,
            },
        }
    }

    pub fn load(&self) -> Option<RcuReader<T>> {
        self.inner.get()
    }

    pub fn try_lock(&self) -> Option<RcuGuard<T>> {
        if self.inner.acquire() {
            return Some(RcuGuard {
                wrapper: &self.inner,
            });
        }
        None
    }

    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_default() {
        let x = RcuCell::<u32>::default();
        assert!(x.load().is_none());
    }

    #[test]
    fn simple_drop() {
        let _ = RcuCell::new(Some(10));
    }

    #[test]
    fn single_thread() {
        let t = RcuCell::new(Some(10));
        let x = t.load();
        let y = t.load();
        t.try_lock().unwrap().update(None);
        let z = t.load();
        let a = z.clone();
        drop(t);
        assert_eq!(x.map(|v| *v), Some(10));
        assert_eq!(y.map(|v| *v), Some(10));
        assert_eq!(z.map(|v| *v), None);
        assert_eq!(a.map(|v| *v), None);
    }

    #[test]
    fn single_thread_clone() {
        let t = Arc::new(RcuCell::new(Some(10)));
        let t1 = t.clone();
        assert!(t1.load().map(|v| *v) == Some(10));
        t1.try_lock().unwrap().update(Some(5));
        assert!(t.load().map(|v| *v) == Some(5));
    }

    #[test]
    fn test_rcu_guard() {
        let t = RcuCell::new(Some(10));
        let x = t.load().map(|v| *v);
        let mut g = t.try_lock().unwrap();
        let y = x.map(|v| v + 1);
        g.update(y);
        assert!(t.try_lock().is_none());
        drop(g);
        assert_eq!(t.load().map(|v| *v), Some(11));
    }

    #[test]
    fn test_is_none() {
        let t = RcuCell::new(Some(10));
        assert!(!t.is_none());
        t.try_lock().unwrap().update(None);
        assert!(t.is_none());
    }

    #[test]
    fn test_is_locked() {
        let t = RcuCell::new(Some(10));
        assert!(!t.is_locked());
        let mut g = t.try_lock().unwrap();
        g.update(None);
        assert!(t.is_locked());
        drop(g);
        assert!(!t.is_locked());
    }

    #[test]
    fn test_clone_rcu_cell() {
        let t = Arc::new(RcuCell::new(Some(10)));
        let t1 = t.clone();
        let t2 = t.clone();
        let t3 = t.clone();
        t1.try_lock().unwrap().update(Some(11));
        drop(t1);
        assert_eq!(t.load().map(|v| *v), Some(11));
        t2.try_lock().unwrap().update(Some(12));
        drop(t2);
        assert_eq!(t.load().map(|v| *v), Some(12));
        t3.try_lock().unwrap().update(Some(13));
        drop(t3);
        assert_eq!(t.load().map(|v| *v), Some(13));
    }

    #[test]
    fn test_rcu_reader() {
        let t = Arc::new(RcuCell::new(Some(10)));
        let t1 = t.clone();
        let t2 = t.clone();
        let t3 = t;
        let d1 = t1.load().unwrap();
        let d3 = t3.load().unwrap();
        let mut g = t1.try_lock().unwrap();
        g.update(Some(11));
        drop(g);
        let d2 = t2.load().unwrap();
        assert_ne!(d1, d2);
        assert_eq!(d1, d3);
        assert_ne!(d2, d3);
    }
}
