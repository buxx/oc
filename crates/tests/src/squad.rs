use bon::Builder;
use glam::Vec2;
use oc_individual::{
    IndividualIndex,
    order::Order,
    squad::{Squad, SquadFormation},
};
use oc_root::side::Side;

#[derive(Debug, Builder)]
pub struct TestSquadBuilder {
    #[builder(default = Side::A)]
    side: Side,
    members: Vec<IndividualIndex>,
    #[builder(default = Vec2::new(0., 0.))]
    position: Vec2,
    #[builder(default = SquadFormation::Line)]
    formation: SquadFormation,
    #[builder(default)]
    orders: Vec<Order>,
}

impl TestSquadBuilder {
    pub fn make(self) -> Squad {
        Squad {
            side: self.side,
            actives: self.members.len() as u8,
            members: self.members,
            formation: self.formation,
            orders: self.orders,
            position: [self.position[0], self.position[1]],
        }
    }
}
