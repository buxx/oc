use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_root::WcfgFrom;
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
    commands: Commands,
) {
    let_some!(g = &g.0, return);
    let (i, update) = (event.0, &event.1);
    let_some!(region = world.squads_refs.get(&i).cloned(), return);

    for event in update_squad(g, &mut world, i, region, update) {
        commands.trigger(event);
    }
}

enum UpdateSquadEffect {
    RefreshSquadsOrders(RefreshSquadsOrdersEvent),
}

fn update_squad(
    g: &oc_network::GameConfig,
    world: &mut World,
    i: oc_individual::squad::SquadIndex,
    region: WorldRegionIndex,
    update: &Update,
) -> Vec<UpdateSquadEffect> {
    // Update can have modified region of squad
    let (new_region, events) = {
        let_some!(squads = world.squads.get_mut(&region), return);
        let_some!(squad = squads.get_mut(&i), return);

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
                let now_tile = TileXy::from_(*position, &g.w);
                let now_region = RegionXy::from_(now_tile, &g.w);
                let now_region = WorldRegionIndex::from_(now_region, &g.w);
                // FIXME BS NOW: update worl.squads_refs
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
