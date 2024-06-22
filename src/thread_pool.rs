use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
type Job = Box<dyn FnOnce() + Send + 'static>;
struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        println!("Creating worker {}", id);
        let thread_handle = std::thread::spawn(move || loop {
            println!("Worker {} is running a job", id);
            let message = receiver
                .lock()
                .expect("Could not lock on receiver. This could be a poisoned.")
                .recv();
            match message {
                Ok(Message::NewJob(job)) => {
                    println!("Recieved new job");
                    job();
                }
                Ok(Message::Terminate) => {
                    println!("Received Terminate message for worker {}", id);
                    break;
                }
                Err(err) => {
                    println!("Got receive error for worker {}:{:?}", id, err);
                }
            }
        });
        Worker {
            id,
            thread: Some(thread_handle),
        }
    }
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}
enum Message {
    NewJob(Job),
    Terminate,
}
impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        println!("Creating thread pool");
        let (sender, receiver) = std::sync::mpsc::channel();
        let sender = Some(sender);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_workers);
        for i in 0..num_workers {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        println!("Sending a task");
        self.sender
            .as_ref()
            .expect("No sender initialized for thread pool")
            .send(Message::NewJob(Box::new(f)))
            .expect("Error sending the job");
        println!("Sent a task in the channel");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .as_ref()
                .unwrap()
                .send(Message::Terminate)
                .expect("Error sending the terminate message on channel");
        }
        for worker in &mut self.workers {
            println!("Stopping thread: {}", worker.id);
            if let Some(worker) = worker.thread.take() {
                worker.join().unwrap();
            }
        }
    }
}
