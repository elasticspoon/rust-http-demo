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
            loop {
                let message = reader.lock().expect("could not accquire lock").recv();
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job. Executing...");
                        job();
                    }
                    Err(_) => {
                        eprintln!("Channel is closed...");
                        break;
                    }
                };
            }
        });
        Worker { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    chan: Option<mpsc::Sender<Job>>,
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
            chan: Some(writer),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let _ = self
            .chan
            .as_ref()
            .unwrap()
            .send(job)
            .map_err(|err| eprintln!("Failed to execute job: {}", err));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.chan.take());

        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}...", worker.id);
            worker.thread.join().unwrap();
        }
    }
}
