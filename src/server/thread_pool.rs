use super::errors::ThreadPoolError;
use log::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(dead_code)]
struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, ThreadPoolError> {
        if size == 0 {
            return Err(ThreadPoolError::CreationError(
                "Thread pool size cannot be 0".to_string(),
            ));
        }

        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));

        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        for worker_id in 0..size {
            let worker = Worker::new(worker_id, Arc::clone(&receiver));
            workers.push(worker);
        }

        Ok(ThreadPool {
            workers,
            sender: tx,
        })
    }

    pub fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(closure);
        let _ = self.sender.send(job); // This should never throw an error
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            match receiver.lock() {
                Ok(rec) => {
                    if let Ok(job) = rec.recv() {
                        job();
                    } else {
                        debug!("Worker {} has been terminated", id);
                        break;
                    }
                }
                Err(_) => {
                    error!("Error trying lock mutex, another thread holding the lock panicked");
                }
            }
        });
        Worker { id, handle }
    }
}
