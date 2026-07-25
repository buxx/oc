use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::let_some;
use rustc_hash::FxHashMap;

use crate::ingame::behavior::RefreshSquadsOrdersEvent;
use crate::ingame::input::individual::UpdateSquadEvent;
use crate::states::GameConfig;
use crate::world::World;
use oc_individual::squad::Update;

pub fn on_update_squad(
    event: On<UpdateSquadEvent>,
    g: Res<GameConfig>,
    mut world: ResMut<World>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let (i, update) = (event.0, &event.1);
    let_some!(region = world.squads_refs.get(&i).cloned(), return);

    for event in update_squad(&g.w, &mut world, i, region, update) {
        event.trigger(&mut commands)
    }
}

#[derive(Debug, PartialEq)]
enum UpdateSquadEffect {
    RefreshSquadsOrders(RefreshSquadsOrdersEvent),
}

impl UpdateSquadEffect {
    pub fn trigger(self, commands: &mut Commands) {
        match self {
            UpdateSquadEffect::RefreshSquadsOrders(event) => commands.trigger(event),
        }
    }
}

fn update_squad(
    w: &WorldConfig,
    world: &mut World,
    i: oc_individual::squad::SquadIndex,
    region: WorldRegionIndex,
    update: &Update,
) -> Vec<UpdateSquadEffect> {
    // Update can have modified region of squad
    let (new_region, events) = {
        let_some!(squads = world.squads.get_mut(&region), return vec![]);
        let_some!(squad = squads.get_mut(&i), return vec![]);

        match update {
            Update::SetOrders(orders) => {
                squad.orders = orders.clone();
                (
                    None,
                    vec![UpdateSquadEffect::RefreshSquadsOrders(
                        RefreshSquadsOrdersEvent(i, orders.clone()),
                    )],
                )
            }
            Update::SetPosition(position) => {
                let now_tile = TileXy::from_(*position, w);
                let now_region = RegionXy::from_(now_tile, w);
                let now_region = WorldRegionIndex::from_(now_region, w);
                squad.position = *position;

                if now_region != region {
                    (Some(now_region), vec![])
                } else {
                    (None, vec![])
                }
            }
            Update::SetActives(actives) => {
                squad.actives = *actives;
                (None, vec![])
            }
            Update::Accomplished => (None, vec![]),
        }
    };

    // If squad now in new region
    if let Some(now_region) = new_region {
        // Remove squad from ol region
        if let Some(squads) = world.squads.get_mut(&region) {
            if let Some(squad) = squads.remove(&i) {
                // And put it in new region
                world
                    .squads
                    .entry(now_region)
                    .or_insert_with(|| FxHashMap::default())
                    .insert(i, squad.clone());
            }
        }

        world.squads_refs.insert(i, now_region);
    }

    events
}

#[cfg(test)]
mod tests {
    use crate::tests::world::TestWorld;

    use super::*;
    use ::tests::squad::TestSquad;
    use oc_individual::squad::SquadIndex;
    use oc_root::{WorldConfig, physics::Meters};

    #[test]
    fn test_update_squad_change_region() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .geo_pixels_per_tile(5)
            .region_width(10)
            .region_height(10);
        let squad = TestSquad::builder()
            .position(Vec2::new(0., 0.))
            .members(vec![oc_individual::IndividualIndex(0)])
            .build()
            .make();
        let mut world = TestWorld::builder()
            .squads(vec![(SquadIndex(0), squad)])
            .build()
            .make(&w);
        let i = SquadIndex(0);
        let region = WorldRegionIndex(0);
        let update = Update::SetPosition([50., 0.]); // According to pixels per tile and region size, new region will be region 1

        // When
        let _ = update_squad(&w, &mut world, i, region, &update);

        // Then
        // No more squad in region 0
        assert_eq!(
            world
                .squads
                .get(&WorldRegionIndex(0))
                .cloned()
                .unwrap_or_default()
                .keys()
                .into_iter()
                .collect::<Vec<&SquadIndex>>(),
            Vec::<&SquadIndex>::new(),
        );
        // Squad is in region 1
        assert_eq!(
            world
                .squads
                .get(&WorldRegionIndex(1))
                .cloned()
                .unwrap_or_default()
                .keys()
                .into_iter()
                .collect::<Vec<&SquadIndex>>(),
            vec![&SquadIndex(0)],
        );
        // squad_refs updated
        assert_eq!(
            world.squads_refs.get(&SquadIndex(0)),
            Some(&WorldRegionIndex(1))
        );
    }
}
