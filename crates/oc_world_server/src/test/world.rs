use bon::Builder;
use oc_individual::squad::Squad;

#[derive(Debug, Default, Builder)]
pub struct TestWorld {
    squads: Vec<Squad>,
}

impl TestWorld {
    pub fn squads(mut self, value: Vec<Squad>) -> Self {
        self.squads = value;
        self
    }

    pub fn make(&self) -> oc_world::World {
        todo!()
    }
}
