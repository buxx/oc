use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_root::WcfgFrom;
use rustc_hash::FxHashMap;

use crate::ingame::behavior::RefreshSquadsOrdersEvent;
use crate::ingame::input::individual::UpdateSquadEvent;
use crate::states::GameConfig;
use crate::world::World;
use oc_individual::squad::Update;

pub struct SquadPlugin;

impl Plugin for SquadPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_squad);
    }
}

pub fn on_update_squad(
    event: On<UpdateSquadEvent>,
    g: Res<GameConfig>,
    mut world: ResMut<World>,
    mut commands: Commands,
) {
    let Some(g) = &g.0 else { return };
    let (i, update) = (event.0, &event.1);
    let Some(region) = world.squads_refs.get(&i).cloned() else {
        return;
    };

    // Update can have modified region of squad
    let now_region = {
        let Some(squads) = world.squads.get_mut(&region) else {
            return;
        };
        let Some(squad) = squads.get_mut(&i) else {
            return;
        };

        match update {
            Update::SetOrders(orders) => {
                squad.orders = orders.clone();
                commands.trigger(RefreshSquadsOrdersEvent(i, orders.clone()));
                None
            }
            Update::SetPosition(position) => {
                let now_tile = TileXy::from_(*position, &g.w);
                let now_region = RegionXy::from_(now_tile, &g.w);
                let now_region = WorldRegionIndex::from_(now_region, &g.w);
                // FIXME BS NOW: update worl.squads_refs
                squad.position = *position;

                if now_region != region {
                    Some(now_region)
                } else {
                    None
                }
            }
            Update::Accomplished => None,
        }
    };

    // If squad now in new region
    if let Some(now_region) = now_region {
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
}
