use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    time::Duration,
};

/// Time a runner thread spent sleeping vs. time it was given (sum of tick intervals), since the
/// last report.
#[derive(Debug, Default)]
struct Load {
    slept_us: AtomicU64,
    total_us: AtomicU64,
    /// Last computed percent (f32 bits), kept when a window has no data
    last: AtomicU32,
}

#[derive(Debug, Default)]
pub struct Perf {
    pub individual_ticks: AtomicU64,
    pub visibilities_ticks: AtomicU64,
    pub squad_ticks: AtomicU64,
    pub physic_ticks: AtomicU64,
    /// Load of each thread, by runner name
    loads: Mutex<BTreeMap<&'static str, Vec<Load>>>,
}

impl Perf {
    pub fn individuals_ticks(&self) -> u64 {
        self.individual_ticks.load(Ordering::Relaxed)
    }

    pub fn increment_individual(&self) {
        self.individual_ticks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_visibilities(&self) {
        self.visibilities_ticks.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(unused)]
    pub fn squads_ticks(&self) -> u64 {
        self.squad_ticks.load(Ordering::Relaxed)
    }

    pub fn increment_squad(&self) {
        self.squad_ticks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn physics_ticks(&self) -> u64 {
        self.physic_ticks.load(Ordering::Relaxed)
    }

    pub fn increment_physic(&self) {
        self.physic_ticks.fetch_add(1, Ordering::Relaxed);
    }

    /// Declare `threads` threads for runner `name`
    pub fn init_load(&self, name: &'static str, threads: usize) {
        self.loads
            .lock()
            .expect("Assume available")
            .insert(name, (0..threads).map(|_| Load::default()).collect());
    }

    /// Record that thread `i` of runner `name` slept `wait` of its tick `interval`.
    /// Sleeping the whole interval is 0% load, not sleeping at all is 100%.
    pub fn record_load(&self, name: &'static str, i: usize, wait: Duration, interval: Duration) {
        if let Some(load) = self.loads.lock().expect("Assume available")[name].get(i) {
            load.slept_us
                .fetch_add(wait.as_micros() as u64, Ordering::Relaxed);
            load.total_us
                .fetch_add(interval.as_micros() as u64, Ordering::Relaxed);
        }
    }

    /// Mean load in percent of the threads of each runner since last call. A thread without
    /// data since last call keeps its previous value.
    pub fn take_loads(&self) -> Vec<(&'static str, f32)> {
        self.loads
            .lock()
            .expect("Assume available")
            .iter()
            .filter(|(_, loads)| !loads.is_empty())
            .map(|(name, loads)| {
                let sum: f32 = loads
                    .iter()
                    .map(|load| {
                        let slept = load.slept_us.swap(0, Ordering::Relaxed);
                        let total = load.total_us.swap(0, Ordering::Relaxed);
                        if total > 0 {
                            let percent = (1. - slept as f32 / total as f32).clamp(0., 1.) * 100.;
                            load.last.store(percent.to_bits(), Ordering::Relaxed);
                        }
                        f32::from_bits(load.last.load(Ordering::Relaxed))
                    })
                    .sum();
                (*name, sum / loads.len() as f32)
            })
            .collect()
    }

    pub fn reset(&self) {
        self.individual_ticks.swap(0, Ordering::Relaxed);
        self.physic_ticks.swap(0, Ordering::Relaxed);
    }
}
