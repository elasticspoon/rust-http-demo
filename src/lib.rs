use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

struct Job {
    closure: fn() -> (),
}

impl Worker {
    fn new(id: usize) -> Worker {
        let thread = thread::spawn(|| {});
        Worker { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    chan: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// # Panics
    ///
    /// The `new` function will panic is size is less than 1
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (writer, _) = mpsc::channel();
        let workers = (0..size).map(Worker::new).collect();
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
