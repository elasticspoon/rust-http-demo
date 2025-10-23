use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

struct Worker<'a> {
    id: usize,
    receiver: &'a mpsc::Receiver<()>,
    thread: JoinHandle<()>,
}

struct Job {
    closure: fn() -> (),
}

impl<'a> Worker<'a> {
    fn new(id: usize, receiver: &Receiver<()>) -> Worker {
        let thread = thread::spawn(|| {});
        Worker {
            id,
            thread,
            receiver,
        }
    }
}

pub struct ThreadPool<'a> {
    workers: Vec<Worker<'a>>,
    chan: mpsc::Sender<()>,
}

impl<'a> ThreadPool<'a> {
    /// Create a new ThreadPool
    ///
    /// # Panics
    ///
    /// The `new` function will panic is size is less than 1
    pub fn new(size: usize) -> ThreadPool<'a> {
        assert!(size > 0);

        let (writer, reader) = mpsc::channel();
        let workers = (0..size).map(move |id| Worker::new(id, &reader)).collect();
        ThreadPool {
            workers,
            chan: writer,
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
