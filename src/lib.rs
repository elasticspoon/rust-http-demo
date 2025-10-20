use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

struct Worker {
    id: usize,
    receiver: mpsc::Receiver<()>,
    thread: JoinHandle<()>,
}

struct Job {
    closure: fn() -> (),
}

impl Worker {
    fn new(id: usize, receiver: Receiver<()>) -> Worker {
        let thread = thread::spawn(|| {});
        Worker {
            id,
            thread,
            receiver,
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    chan: mpsc::Sender<()>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// # Panics
    ///
    /// The `new` function will panic is size is less than 1
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (writer, reader) = mpsc::channel();
        let workers = (0..size).map(|id| Worker::new(id, reader)).collect();
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
