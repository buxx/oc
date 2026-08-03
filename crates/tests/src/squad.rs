use bon::Builder;
use oc_individual::{
    IndividualIndex,
    order::Order,
    squad::{Squad, SquadFormation},
};
use oc_root::{geo::WorldVec2, side::Side};

#[derive(Debug, Builder)]
pub struct TestSquad {
    #[builder(default = Side::A)]
    side: Side,
    members: Vec<IndividualIndex>,
    #[builder(default = WorldVec2::new(0., 0.))]
    position: WorldVec2,
    #[builder(default = SquadFormation::Line)]
    formation: SquadFormation,
    #[builder(default)]
    orders: Vec<Order>,
}

impl TestSquad {
    pub fn make(self) -> Squad {
        Squad {
            side: self.side,
            actives: self.members.len() as u8,
            members: self.members,
            formation: self.formation,
            orders: self.orders,
            position: self.position,
        }
    }
}
