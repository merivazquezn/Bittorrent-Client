use super::errors::ThreadPoolError;
use log::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Struct representing the thread pool
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<ThreadPoolMessage>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum ThreadPoolMessage {
    ExecuteJob(Job),
    Stop,
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    pub handle: JoinHandle<()>,
}

impl ThreadPool {
    /// Creates a new thread pool with the given number of workers.
    /// There will be 'size' workers running at the same time, where each worker repeats retreives jobs from a queue and executes them
    /// The job queue is protected by a mutex, so only one worker can access it at a time.
    ///
    /// # Arguments
    /// * `size` - The number of workers to create.
    ///
    /// # Returns
    ///
    /// #On success
    /// A new thread pool, of type `ThreadPool`.
    ///
    /// #On error
    /// A `ThreadPoolError`, with the underlying cause of failure
    ///
    /// # Example
    /// ```no_run
    ///
    /// use bittorrent_rustico::server::ThreadPool;
    ///
    /// fn shout(message: &str) {
    ///     let loud_message = message.to_uppercase();
    ///     println!("{}", loud_message);
    /// }
    ///
    /// let worker_count = 10;
    /// let pool: ThreadPool = ThreadPool::new(worker_count).unwrap();
    ///
    /// pool.execute(|| {
    ///     shout("hello");
    /// });
    ///
    /// pool.execute(|| {
    ///     shout("Bittorrent rustico is the new bittorrent");
    /// });
    ///
    /// pool.stop().unwrap();
    /// ```
    ///
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

    /// Queues a job to be eventually executed by a threadpool worker.
    ///
    /// # Arguments
    /// * `closure` - The job to be executed, it is a closure with the properties defined as 'FnOnce() + Send + 'static'
    ///
    /// # Example
    ///  Check the 'new' method example
    ///
    pub fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(closure);
        let _ = self.sender.send(ThreadPoolMessage::ExecuteJob(job));
    }

    /// Stops the threadpool
    /// All workers threads are joined
    /// The workers that are executing a job will not be interrupted, so this method will wait all current jobs to end
    /// In order to finish it
    ///
    /// # Returns
    ///
    /// ## On success
    /// Ok(())
    ///
    /// ## On error
    /// A ´ThreadPoolError´ with the underlying cause of the failure
    ///
    /// # Example
    /// Check the 'new' method example
    ///
    pub fn stop(self) -> Result<(), ThreadPoolError> {
        for worker in self.workers {
            self.sender.send(ThreadPoolMessage::Stop).map_err(|_| {
                ThreadPoolError::JoinError(format!("Worker with id {} panicked", worker.id))
            })?;

            worker.handle.join().map_err(|_| {
                ThreadPoolError::JoinError(format!(
                    "Unable to join thread of worker with id: {}",
                    worker.id
                ))
            })?;
        }
        Ok(())
    }
}

impl Worker {
    /// The worker starts running inmediatly after created
    /// In each iteration, the worker tries to grab the job queue lock
    /// When achieved, executes the job, and repeats when finished
    ///
    /// If the message is of type 'stop', the worker is terminated
    ///
    fn new(id: usize, receiver: Arc<Mutex<Receiver<ThreadPoolMessage>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            match receiver.lock() {
                Ok(rec) => {
                    if let Ok(message) = rec.recv() {
                        match message {
                            ThreadPoolMessage::ExecuteJob(job) => {
                                job();
                            }
                            ThreadPoolMessage::Stop => {
                                break;
                            }
                        }
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
