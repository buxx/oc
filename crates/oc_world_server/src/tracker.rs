use std::sync::{Arc, Mutex};

use oc_individual::IndividualIndex;
use oc_physics::Event;

use crate::state::ObjectId;

#[derive(Debug, Clone, Default)]
pub struct Tracker {
    inner: Arc<Mutex<Tracker_>>,
}

impl Tracker {
    pub fn take(&self) -> std::sync::MutexGuard<'_, Tracker_> {
        self.inner
            .lock()
            .expect("Assume tracker is always available")
    }
}

#[derive(Debug, Default)]
pub struct Tracker_ {
    pub physics: Vec<Event<ObjectId>>,
    pub individuals: Vec<(IndividualIndex, oc_individual::Update)>,
}
