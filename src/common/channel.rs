use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{
        AtomicBool,
        Ordering::{Acquire, Relaxed, Release},
    },
    thread::Thread,
};

const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const READING: u8 = 3;

pub struct OneShotChannel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

pub struct Sender<'a, T> {
    channel: &'a OneShotChannel<T>,
    receiving_thread: Thread,
}

pub struct Receiver<'a, T> {
    channel: &'a OneShotChannel<T>,
    _no_send: PhantomData<*const ()>,
}

unsafe impl<T> Sync for OneShotChannel<T> where T: Send {}

impl<T> OneShotChannel<T> {
    const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        *self = Self::new();
        (
            Sender {
                channel: self,
                receiving_thread: std::thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}

impl<T> Sender<'_, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
        self.receiving_thread.unpark();
    }
}

impl<T> Receiver<'_, T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            std::thread::park();
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<T> Drop for OneShotChannel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

#[test]
fn test_one_shot_channel() {
    let mut channel = OneShotChannel::new();
    std::thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("hello world");
        });

        assert_eq!(receiver.receive(), "hello world");
    })
}
