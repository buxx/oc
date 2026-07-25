use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Default)]
pub struct Perf {
    pub individual_ticks: AtomicU64,
    pub individual_percents: Mutex<Vec<f32>>,
    pub squad_ticks: AtomicU64,
    pub squad_percents: Mutex<Vec<f32>>,
    pub physic_ticks: AtomicU64,
    pub physic_percents: Mutex<Vec<f32>>,
}

impl Perf {
    pub fn individuals_ticks(&self) -> u64 {
        self.individual_ticks
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment_individual(&self) {
        self.individual_ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_individual_percent(&self, i: usize, value: f32) {
        self.individual_percents.lock().expect("Assume available")[i] = value;
    }

    pub fn squads_ticks(&self) -> u64 {
        self.squad_ticks.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment_squad(&self) {
        self.squad_ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_squad_percent(&self, i: usize, value: f32) {
        self.squad_percents.lock().expect("Assume available")[i] = value;
    }

    pub fn physics_ticks(&self) -> u64 {
        self.physic_ticks.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment_physic(&self) {
        self.physic_ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_physic_percent(&self, i: usize, value: f32) {
        self.physic_percents.lock().expect("Assume available")[i] = value;
    }

    pub fn reset(&self) {
        self.individual_ticks.swap(0, Ordering::Relaxed);
        self.physic_ticks.swap(0, Ordering::Relaxed);
    }
}
