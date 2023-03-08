use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{
        fence, AtomicUsize,
        Ordering::{Acquire, Relaxed, Release},
    },
};

struct OwnArcData<T> {
    data_ref_count: AtomicUsize,
    alloc_ref_count: AtomicUsize,
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Weak<T> {
    ptr: NonNull<OwnArcData<T>>,
}

unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

pub struct OwnArc<T> {
    ptr: NonNull<OwnArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for OwnArc<T> {}
unsafe impl<T: Send + Sync> Sync for OwnArc<T> {}

impl<T> Weak<T> {
    fn data(&self) -> &OwnArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn upgrade(&self) -> Option<OwnArc<T>> {
        let mut n = self.data().data_ref_count.load(Relaxed);
        loop {
            if n == 0 {
                return None;
            }
            assert!(n <= usize::MAX / 2);

            if let Err(e) =
                self.data()
                    .data_ref_count
                    .compare_exchange_weak(n, n + 1, Relaxed, Relaxed)
            {
                n = e;
                continue;
            }

            return Some(OwnArc { ptr: self.ptr });
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);

            unsafe { Box::from_raw(self.ptr.as_ptr()) };
        }
    }
}

impl<T> OwnArc<T> {
    pub fn new(data: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(OwnArcData {
                data_ref_count: AtomicUsize::new(1),
                alloc_ref_count: AtomicUsize::new(1),
                data: UnsafeCell::new(ManuallyDrop::new(data)),
            }))),
        }
    }

    fn data(&self) -> &OwnArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc
            .data()
            .alloc_ref_count
            .compare_exchange(1, usize::MAX, Acquire, Relaxed)
            .is_err()
        {
            return None;
        }
        let is_unique = arc.data().data_ref_count.load(Relaxed) == 1;

        arc.data().alloc_ref_count.store(1, Release);

        if !is_unique {
            return None;
        }
        fence(Acquire);

        unsafe { Some(&mut *arc.data().data.get()) }
    }

    pub fn downgrade(arc: &Self) -> Weak<T> {
        let mut n = arc.data().alloc_ref_count.load(Relaxed);

        loop {
            if n == usize::MAX {
                std::hint::spin_loop();
                n = arc.data().alloc_ref_count.load(Relaxed);
                continue;
            }
            assert!(n <= usize::MAX / 2);

            if let Err(e) =
                arc.data()
                    .alloc_ref_count
                    .compare_exchange_weak(n, n + 1, Acquire, Relaxed)
            {
                n = e;
                continue;
            }
            return Weak { ptr: arc.ptr };
        }
    }
}

impl<T> Deref for OwnArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data().data.get() }
    }
}

impl<T> Clone for OwnArc<T> {
    fn clone(&self) -> Self {
        if self.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }

        Self { ptr: self.ptr }
    }
}

impl<T> Drop for OwnArc<T> {
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);

            unsafe {
                ManuallyDrop::drop(&mut *self.data().data.get());
            }
            drop(Weak { ptr: self.ptr })
        }
    }
}

#[test]
fn test_own_arc() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Relaxed);
        }
    }

    // Create an Arc with two weak pointers.
    let x = OwnArc::new(("hello", DetectDrop));
    let y = OwnArc::downgrade(&x);
    let z = OwnArc::downgrade(&x);

    let t = std::thread::spawn(move || {
        // Weak pointer should be upgradable at this point.
        let y = y.upgrade().unwrap();
        assert_eq!(y.0, "hello");
    });
    assert_eq!(x.0, "hello");
    t.join().unwrap();

    // The data shouldn't be dropped yet,
    // and the weak pointer should be upgradable.
    assert_eq!(NUM_DROPS.load(Relaxed), 0);
    assert!(z.upgrade().is_some());

    drop(x);

    // Now, the data should be dropped, and the
    // weak pointer should no longer be upgradable.
    assert_eq!(NUM_DROPS.load(Relaxed), 1);
    assert!(z.upgrade().is_none());
}
