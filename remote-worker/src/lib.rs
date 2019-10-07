use error::RemoteError;
use std::sync::mpsc;
use std::thread;

#[derive(Clone)]
pub enum WorkerStatus {
    Running,
    Resolved,
    Rejected,
}

pub struct Worker {
    process: thread::JoinHandle<Result<(String, String), RemoteError>>,
    receiver: mpsc::Receiver<WorkerStatus>,
    status: WorkerStatus,
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
            status: WorkerStatus::Running,
        }
    }

    pub fn get_status(&mut self) -> WorkerStatus {
        if let WorkerStatus::Running = self.status {
            if let Ok(s) = self.receiver.try_recv() {
                self.status = s;
            }
        }
        self.status.clone()
    }

    pub fn join(self) -> Result<(String, String), RemoteError> {
        self.process.join().unwrap()
    }
}
