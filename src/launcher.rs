use crate::{error::Result, Config, PathBuffer, Task};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Launcher {
    tasks: Vec<Task>,
    start_time: Instant,
    pub active: Arc<AtomicBool>,
}

impl<'a> Launcher {
    #[inline]
    pub fn new(config: &mut Config, log_dir: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            tasks: config.generate_tasks(log_dir.map(PathBuffer::from))?,
            start_time: Instant::now(),
            active: Arc::new(AtomicBool::new(true)),
        })
    }

    #[inline]
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn run(&mut self) -> Result<()> {
        self.start()?;

        let active = Arc::clone(&self.active);
        ctrlc::set_handler(move || active.store(false, Ordering::Relaxed))?;

        let active = Arc::clone(&self.active);
        while active.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }

        self.shutdown()
    }

    fn start(&mut self) -> Result<()> {
        self.tasks.iter_mut().try_for_each(|task| task.spawn())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.tasks.iter_mut().try_for_each(|task| task.kill())
    }
}
