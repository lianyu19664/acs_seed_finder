use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
};

#[derive(Default)]
pub struct BackgroundTask<T> {
    pub is_running: bool,
    pub progress: Arc<AtomicUsize>,
    pub total_tasks: usize,
    rx: Option<mpsc::Receiver<T>>,
}
impl<T: Send + 'static> BackgroundTask<T> {
    pub fn start<F: FnOnce(Arc<AtomicUsize>) -> T + Send + 'static>(&mut self, t: usize, f: F) {
        self.is_running = true;
        self.total_tasks = t;
        self.progress.store(0, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let p = self.progress.clone();
        thread::spawn(move || {
            let _ = tx.send(f(p));
        });
    }
    pub fn poll(&mut self) -> Option<Result<T, mpsc::TryRecvError>> {
        let res = self.rx.as_ref()?.try_recv();
        match res {
            Ok(r) => {
                self.is_running = false;
                self.rx = None;
                Some(Ok(r))
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(e) => {
                self.is_running = false;
                self.rx = None;
                Some(Err(e))
            }
        }
    }
    pub fn fraction(&self) -> f32 {
        if self.total_tasks > 0 {
            (self.progress.load(Ordering::Relaxed) as f32 / self.total_tasks as f32).clamp(0., 1.)
        } else {
            0.
        }
    }
}
