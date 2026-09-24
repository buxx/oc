use std::collections::HashMap;

use getset::WithSetters;
use oc_root::geo::WorldVec2;
use uuid::Uuid;

#[derive(Debug, WithSetters)]
pub struct Placer {
    places: HashMap<Uuid, WorldVec2>,
    #[getset(set_with = "pub")]
    order: OrderPolicy,
}

impl Placer {
    pub fn random() -> Self {
        Self {
            places: HashMap::default(),
            order: OrderPolicy::default(),
        }
    }
    pub fn places(places: Vec<(Uuid, WorldVec2)>) -> Self {
        Self {
            places: places.into_iter().collect(),
            order: OrderPolicy::default(),
        }
    }

    pub fn place(&self, uuid: Uuid) -> WorldVec2 {
        if let Some(place) = self.places.get(&uuid) {
            return *place;
        }

        // FIXME BS NOW: place somewhere (must know spawn zones, etc)
        todo!()
    }

    pub fn orders(
        &self,
        _w: &oc_root::WorldConfig,
        _mod_: &oc_mod::Mod,
        snapshot: &oc_world::snapshot::Snapshot,
        squad: &oc_individual::squad::Squad,
    ) -> Vec<oc_individual::order::Order> {
        match self.order {
            OrderPolicy::None => vec![],
            OrderPolicy::Hide => {
                let opposite_side = squad.side.opposite();
                let nearest = snapshot
                    .individuals
                    .iter()
                    .filter(|individual| individual.side == opposite_side)
                    .min_by(|a, b| {
                        let da = WorldVec2::from(a.position) - squad.position;
                        let db = WorldVec2::from(b.position) - squad.position;
                        (da.x * da.x + da.y * da.y).total_cmp(&(db.x * db.x + db.y * db.y))
                    });

                let direction = match nearest {
                    Some(individual) => oc_utils::d2::Direction::from_points2d(
                        squad.position.into(),
                        WorldVec2::from(individual.position).into(),
                    ),
                    None => oc_utils::d2::Direction::default(),
                };

                vec![oc_individual::order::Order::Hide(direction)]
            }
        }
    }
}

#[derive(Debug, Default)]
pub enum OrderPolicy {
    None,
    #[default]
    Hide,
}
