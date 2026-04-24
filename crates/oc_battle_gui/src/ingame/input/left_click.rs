#[cfg(feature = "debug")]
use crate::ingame::debug::projectile::SpawnProjectileProfile;
#[cfg(feature = "debug")]
use crate::window::debug::battle::SpawnProjectileClickMode;
#[cfg(feature = "debug")]
use crate::world::World;
#[cfg(feature = "debug")]
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
#[cfg(feature = "debug")]
use oc_geo::tile::TileXy;
#[cfg(feature = "debug")]
use oc_network::ToServer;
#[cfg(feature = "debug")]
use oc_projectile::spawn::SpawnProjectile;
#[cfg(feature = "debug")]
use strum_macros::EnumIter;

#[cfg(feature = "debug")]
use crate::window::PointerInWindow;
#[cfg(feature = "debug")]
use crate::{ingame::draw, network::output::ToServerEvent};

#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetLeftClick(pub LeftClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetSpawnProjectileLeftClickMode(pub SpawnProjectileClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Event)]
pub struct SpawnClicksLine;

#[cfg(feature = "debug")]
#[derive(Debug, Event)]
pub struct DespawnClicksLine;

#[cfg(feature = "debug")]
#[derive(Debug, Component)]
pub struct ClicksLine;

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct LeftClick(pub LeftClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct SpawnProjectileLeftClick(pub SpawnProjectileClickMode);

#[derive(Debug, Clone, Default)]
pub enum LeftClickMode {
    #[default]
    Select,
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectileProfile),
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone, Default, PartialEq, Eq, EnumIter)]
pub enum LeftClickModeType {
    #[default]
    Select,
    #[cfg(feature = "debug")]
    SpawnProjectile,
}

#[cfg(feature = "debug")]
impl LeftClickModeType {
    pub fn name(&self) -> &str {
        match self {
            LeftClickModeType::Select => "Select",
            LeftClickModeType::SpawnProjectile => "Spawn projectile",
        }
    }
}

#[cfg(feature = "debug")]
pub fn click_debug(
    mut commands: Commands,
    ignore: Res<PointerInWindow>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    spawn_mode: Res<SpawnProjectileLeftClick>,
    _keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<super::State>,
    world: Res<World>,
) {
    if ignore.0 {
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (camera, transform) = *camera;
    let Ok(point) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

    match &mode.0 {
        LeftClickMode::Select => {
            // TODO
        }
        #[cfg(feature = "debug")]
        LeftClickMode::SpawnProjectile(profile) => match spawn_mode.0 {
            SpawnProjectileClickMode::TwoClicks => {
                if buttons.just_released(MouseButton::Left) {
                    state.clicks.push(point);

                    if state.clicks.len() == 1 {
                        commands.trigger(SpawnClicksLine);
                    }

                    // TODO: refactor (see bellow)
                    if state.clicks.len() == 2 {
                        use oc_root::{GEO_PIXELS_PER_METERS, y::Y};

                        let start = state.clicks.first().expect("len checked line before");
                        let start_tile_xy = TileXy::from((start.x, start.y.to_world_y()));
                        let Some(start_tile) = world.tile(start_tile_xy) else {
                            return;
                        };
                        // FIXME BS NOW: 0.5 must be config ratio (const ?) meters by height ID
                        let start_z = start_tile.z as f32 * 0.5 * GEO_PIXELS_PER_METERS;
                        let start = [start.x, start.y.to_world_y(), start_z];
                        let end = state.clicks.last().expect("len checked line before");
                        let end_tile_xy = TileXy::from((end.x, end.y.to_world_y()));
                        let Some(end_tile) = world.tile(end_tile_xy) else {
                            return;
                        };
                        // FIXME BS NOW: 0.5 must be config ratio (const ?) meters by height ID
                        let end_z = end_tile.z as f32 * 0.5 * GEO_PIXELS_PER_METERS;
                        let end = [end.x, end.y.to_world_y(), end_z];
                        let weapon = profile.weapon;
                        let ammo = profile.ammunition;
                        let shot = profile.shot;
                        let repeat = profile.repeat;
                        let spawn = SpawnProjectile::new(weapon, ammo, shot, repeat, start, end);

                        tracing::debug!("Spawn projectile {spawn:?}");
                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                        commands.trigger(DespawnClicksLine);

                        state.clicks.clear();
                    }
                }
            }
            SpawnProjectileClickMode::DraggedClick => {
                if buttons.just_pressed(MouseButton::Left) {
                    state.clicks.push(point);
                    commands.trigger(SpawnClicksLine);
                }

                // TODO: refactor (see before) into function wich take world xy start, and xy end
                if buttons.just_released(MouseButton::Left) {
                    if let Some(start) = state.clicks.first() {
                        use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
                        use oc_root::{GEO_PIXELS_PER_METERS, y::Y};

                        // tracing::error!("DEBUGG: start={start:?}");
                        // tracing::error!("DEBUGG: y {} => {}", start.y, start.y.to_world_y());
                        let start_ = (start.x, start.y.to_world_y());
                        let start_tile_xy = TileXy::from(start_);
                        // tracing::error!("DEBUGG: start_tile_xy={start_tile_xy:?}");
                        let Some(start_tile) = world.tile(start_tile_xy) else {
                            return;
                        };
                        // tracing::error!(
                        //     "DEBUGG start_={start_:?} start_tile_xy={start_tile_xy:?} start_tile={start_tile:?}"
                        // );
                        // tracing::error!("DEBUGG: start_tile={start_tile:?}");
                        // tracing::trace!(
                        //     name = "debugg",
                        //     tile_z = start_tile.z,
                        //     pp = ?profile.plus_z,
                        //     ppp = profile.plus_z.pixels()
                        // );
                        // FIXME BS NOW: 0.5 must be config ratio (const ?) meters by height ID
                        let start_z = (start_tile.z as f32 * 0.5 * GEO_PIXELS_PER_METERS)
                            + profile.plus_z.pixels();
                        let start = [start.x, start.y.to_world_y(), start_z];

                        // tracing::error!("DEBUGG: end={point:?}");
                        // tracing::error!("DEBUGG: y {} => {}", point.y, point.y.to_world_y());

                        let end_ = (point.x, point.y.to_world_y());
                        let end_tile_xy = TileXy::from(end_);
                        let x: WorldTileIndex = end_tile_xy.into();
                        // tracing::error!("DEBUGG: end_tile_xy={end_tile_xy:?} (==>{x:?})",);
                        let Some(end_tile) = world.tile(end_tile_xy) else {
                            return;
                        };

                        // tracing::error!(
                        //     "DEBUGG end_={end_:?} end_tile_xy={end_tile_xy:?} end_tile={end_tile:?}"
                        // );
                        // dbg!((
                        //     &world.tiles[&WorldRegionIndex(0)][&WorldTileIndex(0)],
                        //     &world.tiles[&WorldRegionIndex(0)][&WorldTileIndex(1)]
                        // ));

                        // tracing::error!("DEBUGG: end_tile={end_tile:?}");
                        // FIXME BS NOW: 0.5 must be config ratio (const ?) meters by height ID
                        let end_z = end_tile.z as f32 * 0.5 * GEO_PIXELS_PER_METERS;
                        let end = [point.x, point.y.to_world_y(), end_z];

                        let weapon = profile.weapon;
                        let ammo = profile.ammunition;
                        let shot = profile.shot;
                        let repeat = profile.repeat;
                        let spawn = SpawnProjectile::new(weapon, ammo, shot, repeat, start, end);

                        // tracing::error!(
                        //     "DEBUGG: start_tile={start_tile:?} end_tile={end_tile:?} start={start:?} end={end:?}"
                        // );
                        tracing::debug!("Spawn projectile {spawn:?}");
                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                    }

                    commands.trigger(DespawnClicksLine);
                    state.clicks.clear();
                }
            }
        },
    }

    //
}

pub fn on_set_left_click(set: On<SetLeftClick>, mut left_click: ResMut<LeftClick>) {
    left_click.0 = set.0.clone();
}

#[cfg(feature = "debug")]
pub fn on_set_spawn_projectile_left_click(
    set: On<SetSpawnProjectileLeftClickMode>,
    mut left_click: ResMut<SpawnProjectileLeftClick>,
) {
    left_click.0 = set.0.clone();
}

#[cfg(feature = "debug")]
pub fn on_spawn_clicks_line(
    _: On<SpawnClicksLine>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<super::State>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (camera, transform) = *camera;
    let Ok(point) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

    let mut points = state.clicks.clone();
    points.push(point);
    let line = Polyline2d::new(points);

    commands.spawn((
        ClicksLine,
        Mesh2d(meshes.add(line)),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::from_xyz(0., 0., draw::Z_SELECT_WIRES),
    ));
}

#[cfg(feature = "debug")]
pub fn update_spawn_projectile_clicks_line(
    mut commands: Commands,
    mode: Res<LeftClick>,
    state: Res<super::State>,
) {
    match &mode.0 {
        LeftClickMode::Select => {}
        LeftClickMode::SpawnProjectile(_) => {
            if !state.clicks.is_empty() {
                commands.trigger(DespawnClicksLine);
                commands.trigger(SpawnClicksLine);
            }
        }
    }
}

#[cfg(feature = "debug")]
pub fn on_despawn_clicks_line(
    _: On<DespawnClicksLine>,
    mut commands: Commands,
    line: Single<Entity, With<ClicksLine>>,
) {
    commands.entity(line.into_inner()).despawn();
}
