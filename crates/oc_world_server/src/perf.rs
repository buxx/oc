use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct Perf {
    pub tick: AtomicU64,
}

impl Perf {
    pub fn ticks(&self) -> u64 {
        self.tick.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn incr(&self) {
        self.tick.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.tick.swap(0, Ordering::Relaxed);
    }
}
