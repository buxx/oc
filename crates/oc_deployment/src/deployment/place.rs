use std::collections::HashMap;

use oc_root::geo::WorldVec2;
use uuid::Uuid;

pub struct Placer {
    places: HashMap<Uuid, WorldVec2>,
}

impl Placer {
    pub fn random() -> Self {
        Self {
            places: HashMap::default(),
        }
    }
    pub fn places(places: Vec<(Uuid, WorldVec2)>) -> Self {
        Self {
            places: places.into_iter().collect(),
        }
    }

    pub fn place(&self, uuid: Uuid) -> WorldVec2 {
        if let Some(place) = self.places.get(&uuid) {
            return *place;
        }

        // FIXME BS NOW: place somewhere (must know spawn zones, etc)
        todo!()
    }
}
