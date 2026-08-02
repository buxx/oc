use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::y::Y;

use crate::ingame::input::left_click::LeftClick;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;

pub fn show(
    w: &WorldConfig,
    point: Vec2,
    commands: &mut Commands,
    mode: &LeftClick,
    ingame: &mut crate::ingame::state::State,
    world: &crate::world::World,
) {
    let spawns = ingame.selected_squads().iter().filter_map(|i| {
        tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
        let squad = world.squad(i)?;
        let leader = world.get_individual(squad.leader())?;
        let start = Vec2::new(leader.position[0], leader.position[1]);
        let start = start.to_gui_y(w);
        let end = point;
        let start_tile = TileXy::from_([start.x, start.y], w);
        let start_tile = WorldTileIndex::from_(start_tile, w);
        let key = SpawnPathProfileKey::Squad{ i: *i, start: start_tile, end };
        Some(SpawnPathProfile { key, start, end })
    }).collect::<Vec<SpawnPathProfile>>();
    commands.trigger(ComputeDisplayPaths(spawns));
}

pub fn cancel(commands: &mut Commands) {
    commands.trigger(ComputeDisplayPaths(vec![]));
}
