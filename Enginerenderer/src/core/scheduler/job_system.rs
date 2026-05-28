use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;

pub struct Job {
    pub task: Box<dyn FnOnce() + Send>,
    pub priority: u8,
}

struct SharedQueue {
    queue: Mutex<VecDeque<Job>>,
    condvar: Condvar,
}

impl SharedQueue {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        })
    }

    fn push(&self, job: Job) {
        let mut q = self.queue.lock().unwrap();
        let pos = q.iter().position(|j| j.priority < job.priority).unwrap_or(q.len());
        q.insert(pos, job);
        self.condvar.notify_one();
    }

    fn pop_or_wait(&self, shutdown: &AtomicBool) -> Option<Job> {
        let mut q = self.queue.lock().unwrap();
        loop {
            if let Some(job) = q.pop_front() {
                return Some(job);
            }
            if shutdown.load(Ordering::Relaxed) {
                return None;
            }
            q = self.condvar.wait(q).unwrap();
        }
    }
}

pub struct JobSystem {
    shared: Arc<SharedQueue>,
    shutdown: Arc<AtomicBool>,
    active_count: Arc<AtomicUsize>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for JobSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobSystem")
            .field("worker_count", &self.workers.len())
            .field("active_count", &self.active_count.load(std::sync::atomic::Ordering::Relaxed))
            .finish()
    }
}

impl JobSystem {
    pub fn new(thread_count: usize) -> Self {
        let shared = SharedQueue::new();
        let shutdown = Arc::new(AtomicBool::new(false));
        let active_count = Arc::new(AtomicUsize::new(0));
        let mut workers = Vec::with_capacity(thread_count);

        for _ in 0..thread_count {
            let shared_clone = Arc::clone(&shared);
            let shutdown_clone = Arc::clone(&shutdown);
            let active_clone = Arc::clone(&active_count);
            workers.push(thread::spawn(move || {
                worker_loop(shared_clone, shutdown_clone, active_clone);
            }));
        }

        Self { shared, shutdown, active_count, workers }
    }

    pub fn spawn<F: FnOnce() + Send + 'static>(&self, task: F, priority: u8) {
        self.shared.push(Job { task: Box::new(task), priority });
    }

    pub fn wait_all(&self) {
        loop {
            let active = self.active_count.load(Ordering::SeqCst);
            let pending = self.shared.queue.lock().unwrap().len();
            if active == 0 && pending == 0 { break; }
            std::hint::spin_loop();
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn pending_count(&self) -> usize {
        self.shared.queue.lock().unwrap().len()
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.shared.condvar.notify_all();
        for handle in self.workers.drain(..) {
            let _ = handle.join();
        }
    }
}

fn worker_loop(shared: Arc<SharedQueue>, shutdown: Arc<AtomicBool>, active_count: Arc<AtomicUsize>) {
    while let Some(job) = shared.pop_or_wait(&shutdown) {
        active_count.fetch_add(1, Ordering::SeqCst);
        (job.task)();
        active_count.fetch_sub(1, Ordering::SeqCst);
    }
}
