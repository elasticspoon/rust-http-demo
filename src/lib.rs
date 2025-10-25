use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, reader: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            let job = reader.lock().unwrap().recv().unwrap();
            println!("Worker {id} got a job. Executing...");
            job();
        });
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

        let (writer, reader) = mpsc::channel();
        let reader = Arc::new(Mutex::new(reader));
        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&reader)))
            .collect();
        ThreadPool {
            workers,
            chan: writer,
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let _ = self
            .chan
            .send(job)
            .map_err(|err| eprintln!("Failed to execute job: {}", err));
    }
}
