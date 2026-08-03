use bevy::prelude::*;
use oc_geo::tile::WorldTileIndex;
use oc_individual::squad::SquadIndex;
use oc_physics::update::bevy::Position;
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::{ScreenPoint2d, WorldPoint2d},
    y::Y,
};
use oc_utils::let_some;

#[cfg(feature = "debug")]
use crate::ingame::camera::squad::ShowFormationPositions;
use crate::{
    entity::individual::{IndividualIndex, Intent},
    ingame::draw,
    states::GameConfig,
    world::World,
};

const PATH_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.15);
#[cfg(feature = "debug")]
const PATH_COLOR_DEBUG: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct PathGizmos;

#[derive(Debug, Resource, Default)]
pub struct DisplayPaths(pub Vec<(SpawnPathProfileKey, Path)>);

#[derive(Debug, Clone, Event, Deref)]
pub struct ComputeDisplayPaths(pub Vec<SpawnPathProfile>);

#[derive(Debug, Clone)]
pub struct SpawnPathProfile {
    pub key: SpawnPathProfileKey,
    pub start: WorldPoint2d,
    pub end: WorldPoint2d,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpawnPathProfileKey {
    Squad {
        i: SquadIndex,
        start: WorldTileIndex,
        end: WorldPoint2d,
    },
}

#[derive(Debug, Clone)]
pub struct Path {
    start: WorldPoint2d,
    segments: Vec<WorldPoint2d>,
}

pub fn on_compute_display_paths(
    profiles: On<ComputeDisplayPaths>,
    g: Res<GameConfig>,
    world: Res<World>,
    mut paths: ResMut<DisplayPaths>,
) {
    let_some!(g = &g.0, return);

    // Empty means remove all
    if profiles.is_empty() {
        tracing::trace!(name = "ingame-input-path-empty");
        paths.0 = vec![];
        return;
    }

    // Avoid computing if not necessary
    let existing: Vec<&SpawnPathProfileKey> = paths.0.iter().map(|p| &p.0).collect();
    let new_ones = profiles.iter().any(|p| !existing.contains(&&p.key));
    let need_compute = !profiles.iter().any(|p| !p.still_valid(&g.w, &world));
    if !new_ones && !need_compute {
        // tracing::trace!(name = "ingame-path-on-compute-display-paths-no-need");
        return;
    }

    let paths_: Vec<(SpawnPathProfileKey, Path)> = profiles
        .iter()
        .filter_map(|profile| {
            let path = world.path(profile.start.into(), profile.end.into());
            let_some!(path = path, return None);
            let segments = path.path.iter().map(|p| [p.x, p.y].into()).collect();
            Some((
                profile.key.clone(),
                Path {
                    start: profile.start,
                    segments,
                },
            ))
        })
        .collect();

    tracing::trace!(name = "ingame-path-on-compute-display-paths", paths_=?paths_);
    paths.0 = paths_;
}

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-behavior-setup-gizmos");
    let (gizmos, _) = config.config_mut::<PathGizmos>();
    gizmos.line.width = 1.0;
    gizmos.line.style = GizmoLineStyle::Dotted;
}

pub fn draw_paths(
    g: Res<GameConfig>,
    intents: Query<(&Intent, &Position), With<IndividualIndex>>,
    display: Res<DisplayPaths>,
    mut gizmos: Gizmos<PathGizmos>,
    #[cfg(feature = "debug")] debug: Res<ShowFormationPositions>,
) {
    let_some!(g = &g.0, return);
    #[cfg(not(feature = "debug"))]
    let color = PATH_COLOR;
    #[cfg(feature = "debug")]
    let color = {
        match debug.0 {
            true => PATH_COLOR_DEBUG,
            false => PATH_COLOR,
        }
    };

    // User giving order paths
    for (_, path) in display.0.iter() {
        let mut previous = ScreenPoint2d::from_(path.start, &g.w);
        // let mut previous: [f32; 2] = [position.0[0], position.0[1]];
        for point in &path.segments {
            let point = ScreenPoint2d::from_(*point, &g.w);
            let start = Vec3::new(previous.x, previous.y, draw::Z_PATH);
            let stop = Vec3::new(point.x, point.y, draw::Z_PATH);
            gizmos.line(start, stop, color);

            previous = point;
        }
    }

    // Individual intent paths
    for (intent, position) in intents {
        match &intent.0 {
            oc_individual::behavior::Intent::Idle(_) => {}
            oc_individual::behavior::Intent::MoveTo(_, path) => {
                let mut previous: [f32; 2] = [position.0[0], position.0[1]];
                for point in path.iter() {
                    let start = Vec3::new(previous[0], previous[1].to_gui_y(&g.w), draw::Z_PATH);
                    let stop = Vec3::new(point[0], point[1].to_gui_y(&g.w), draw::Z_PATH);
                    gizmos.line(start, stop, color);

                    previous = [point[0], point[1]];
                }
            }
        }
    }
}

impl SpawnPathProfile {
    fn still_valid(&self, _w: &WorldConfig, world: &World) -> bool {
        match self.key {
            // Consider valid if squad leader still at same position
            SpawnPathProfileKey::Squad { i, start, end: _ } => {
                let_some!(squad = world.squad(&i), return false);
                let_some!(leader = world.get_individual(squad.leader()), return false);
                start == leader.tile()
            }
        }
    }
}
