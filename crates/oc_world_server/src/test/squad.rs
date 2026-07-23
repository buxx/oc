use bon::Builder;
use glam::Vec3;
use oc_individual::{IndividualIndex, squad::Squad};

#[derive(Debug, Builder)]
pub struct TestSquadBuilder {
    #[builder(default = Vec3::new(0., 0., 0.))]
    position: Vec3,
    members: Vec<IndividualIndex>,
}

impl TestSquadBuilder {
    pub fn make(self) -> Squad {
        Squad {
            side: (),
            members: (),
            actives: (),
            formation: (),
            orders: (),
            position: (),
        }
    }
}
