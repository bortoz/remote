use error::RemoteError;
use std::cell::Cell;
use std::sync::mpsc;
use std::thread;

#[derive(Copy, Clone)]
pub enum WorkerStatus {
    Running,
    Resolved,
    Rejected,
}

pub struct Worker {
    process: thread::JoinHandle<Result<(String, String), RemoteError>>,
    receiver: mpsc::Receiver<WorkerStatus>,
    status: Cell<WorkerStatus>,
}

impl Worker {
    pub fn new<F>(f: F) -> Worker
    where
        F: FnOnce() -> Result<(String, String), RemoteError>,
        F: Send + 'static,
    {
        let (send, recv) = mpsc::channel();
        Worker {
            process: thread::spawn(move || {
                let result = f();
                match result {
                    Ok(_) => send.send(WorkerStatus::Resolved).unwrap(),
                    Err(_) => send.send(WorkerStatus::Rejected).unwrap(),
                };
                result
            }),
            receiver: recv,
            status: Cell::new(WorkerStatus::Running),
        }
    }

    pub fn get_status(&self) -> WorkerStatus {
        if let WorkerStatus::Running = self.status.get() {
            if let Ok(s) = self.receiver.try_recv() {
                self.status.set(s);
            }
        }
        self.status.get()
    }

    pub fn join(self) -> Result<(String, String), RemoteError> {
        self.process.join().unwrap()
    }
}
