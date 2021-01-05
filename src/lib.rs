//! # Hello Web Server
//!
//! `Hello Web Server` is a lib crate that implements an easy way to work with asynchronism,
//! doing this through a pool of threads of configurable size

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// A type alias for a smart pointer of an closure that is executed by workers
type Job = Box<dyn FnOnce() + Send + 'static>;

/// An enum that indicates whether a Worker should run a new Job or shut down
enum Message {
    Newjob(Job),
    Terminate,
}

/// A struct that implements a thread pool.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers: workers,
            sender,
        }
    }

    /// Adds a closure to the pool.
    ///
    /// The closure is encapsulated in a Job that is sent to Workers
    pub fn execute<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(func);

        self.sender.send(Message::Newjob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Send a `Message::Terminate` to all Workers, so that they end their activity.
    ///
    /// Iterates and for each Worker calls the `join()` method on its thread,
    /// ensuring that all Jobs are executed.
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The Worker instance stores a looping thread and blocks until
    /// it receives a `Message`. If it is a `Message::NewJob (job)`,
    /// then it executes the job, and if it is a `Message::Terminate`
    /// the thread is terminated.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::Newjob(job) => {
                    println!("Worker {} got a job. Executing...", id);

                    job();
                }

                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
