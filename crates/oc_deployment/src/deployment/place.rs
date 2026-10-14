use std::collections::HashMap;

use getset::WithSetters;
use oc_root::{geo::WorldVec2, side::Side};
use oc_utils::{d2::Direction, polygon::Polygon};
use oc_world::spawn::SpawnZone;
use uuid::Uuid;

#[derive(Debug, WithSetters)]
pub struct Placer {
    places: Vec<WorldVec2>,
    assigned: HashMap<Uuid, WorldVec2>,
    #[getset(set_with = "pub")]
    a_order: OrderPolicy,
    #[getset(set_with = "pub")]
    b_order: OrderPolicy,
}

impl Placer {
    pub fn new(places: Vec<WorldVec2>) -> Self {
        Self {
            places,
            assigned: HashMap::default(),
            a_order: OrderPolicy::default(),
            b_order: OrderPolicy::default(),
        }
    }

    pub fn with_assigned(mut self, places: Vec<(Uuid, WorldVec2)>) -> Self {
        self.assigned = places.into_iter().collect();
        self
    }

    pub fn place(&self, spawns: &[SpawnZone], uuid: Uuid, occupied: &[WorldVec2]) -> WorldVec2 {
        // If placer already know a place for the squad, us it
        if let Some(place) = self.assigned.get(&uuid) {
            return *place;
        }

        // Elsewhere, find a place in available spawn zone
        if let Some(place) = self
            .places
            .iter()
            .find(|p| !occupied.contains(p) && spawns.iter().any(|s| s.contains(**p)))
        {
            return *place;
        }

        // FIXME BS NOW: place somewhere randomly
        todo!()
    }

    pub fn orders(
        &self,
        _w: &oc_root::WorldConfig,
        _mod_: &oc_mod::Mod,
        snapshot: &oc_world::snapshot::Snapshot,
        squad: &oc_individual::squad::Squad,
    ) -> Vec<oc_individual::order::Order> {
        let policy = match squad.side {
            Side::A => &self.a_order,
            Side::B => &self.b_order,
        };

        if matches!(policy, OrderPolicy::None) {
            return vec![];
        }

        let opposite_side = squad.side.opposite();
        let nearest_enemy = snapshot
            .individuals
            .iter()
            .filter(|individual| individual.side == opposite_side)
            .min_by(|a, b| {
                let da = WorldVec2::from(a.position) - squad.position;
                let db = WorldVec2::from(b.position) - squad.position;
                (da.x * da.x + da.y * da.y).total_cmp(&(db.x * db.x + db.y * db.y))
            });

        let direction = match nearest_enemy {
            Some(individual) => {
                Direction::from_points2d(squad.position.into(), individual.position.into())
            }
            None => oc_utils::d2::Direction::default(),
        };

        vec![match policy {
            OrderPolicy::None => unreachable!(),
            OrderPolicy::Hide => oc_individual::order::Order::Hide(direction),
            OrderPolicy::Defend => oc_individual::order::Order::Defend(direction),
        }]
    }
}

#[derive(Debug, Default)]
pub enum OrderPolicy {
    None,
    #[default]
    Hide,
    Defend,
}
