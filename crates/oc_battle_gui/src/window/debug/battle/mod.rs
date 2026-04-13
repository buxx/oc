use std::fmt::Display;

use bevy::{math::VectorSpace, prelude::*};
use bevy_egui::egui;
use derive_more::Constructor;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_individual::IndividualIndex;
use oc_mod::{
    DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS_PIXELS, Mod,
    ammunition::IndexedAmmunition,
    armament::IndexedShotMode,
    weapons::{IndexedWeapon, WeaponType},
};
use oc_projectile::ProjectileId;
use oc_utils::d2::Xy;
use oc_world::tile::Tile;
use strum_macros::{Display, EnumIter};

pub mod component;
pub mod cursor;
pub mod left_click;
pub mod refresh;
pub mod window;

use crate::{
    ingame::{camera::region::Region, input::left_click::LeftClickModeType},
    window::{MountedWindow, UnmountedWindow, debug::subject::Subject},
};

/// Used to cache debug battle window when not displayed
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DebugBattleWindow(pub Option<window::Window>);

#[derive(Debug, Event)]
pub struct Refresh;

#[derive(Debug, Clone)]
pub struct Context {
    refresh: refresh::Refresh,
    show_tiles: bool,
    // Cursor
    cursor: Option<Vec2>,
    point: Option<Vec2>,
    tile: Option<(TileXy, String)>,
    region: Option<RegionXy>,
    // Components
    view: View,
    regions: Vec<Region>,
    individuals: Vec<Subject<IndividualIndex>>,
    projectiles: Vec<Subject<ProjectileId>>,
    // Left click
    left_click_mode: LeftClickModeType,
    spawn_weapon_type: WeaponType,
    spawn_weapon: Option<IndexedWeapon>,
    spawn_ammunition: Option<IndexedAmmunition>,
    spawn_shot: Option<IndexedShotMode>,
    spawn_projectile_click_mode: SpawnProjectileClickMode,
    spawn_repeat: u8,
    spawn_projectile_plus_z: f32, // pixels
}

impl Default for Context {
    fn default() -> Self {
        Self {
            refresh: Default::default(),
            show_tiles: Default::default(),
            cursor: Default::default(),
            point: Default::default(),
            tile: Default::default(),
            region: Default::default(),
            view: Default::default(),
            regions: Default::default(),
            individuals: Default::default(),
            projectiles: Default::default(),
            left_click_mode: Default::default(),
            spawn_weapon_type: Default::default(),
            spawn_weapon: Default::default(),
            spawn_ammunition: Default::default(),
            spawn_shot: Default::default(),
            spawn_projectile_click_mode: Default::default(),
            spawn_repeat: 1,
            spawn_projectile_plus_z: DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS_PIXELS,
        }
    }
}

#[derive(Constructor)]
pub struct InContext<'a, 'b, 'w, 's> {
    pub context: &'a mut Context,
    pub commands: &'b mut Commands<'w, 's>,
    pub mod_: &'a Mod,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, PartialEq, Eq)]
pub enum SpawnProjectileClickMode {
    TwoClicks,
    #[default]
    DraggedClick,
}

#[derive(Debug, Clone, EnumIter, Default)]
pub enum Tab {
    #[default]
    Cursor,
    Components,
    Leftclick,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Cursor => f.write_str("Cursor"),
            Tab::Components => f.write_str("Components"),
            Tab::Leftclick => f.write_str("Left click"),
        }
    }
}

#[derive(Debug, Clone, EnumIter, Default)]
pub enum View {
    #[default]
    None,
    Regions,
    Individuals,
    Projectiles,
}

impl<'a, 'b, 'w, 's> egui_dock::TabViewer for InContext<'a, 'b, 'w, 's> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let context = &mut self.context;
        let commands = &mut self.commands;
        let mod_ = &self.mod_;

        match tab {
            Tab::Cursor => context.ui_cursor(ui, commands, mod_),
            Tab::Components => context.ui_components(ui, commands, mod_),
            Tab::Leftclick => context.ui_left_click(ui, commands, mod_),
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugBattleWindowPlugin;

impl Plugin for DebugBattleWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugBattleWindow>()
            .add_observer(on_mounted_window)
            .add_observer(on_unmounted_window)
            .add_observer(refresh::on_refresh)
            .add_systems(Update, refresh::trigger_refresh);
    }
}

fn on_mounted_window(mounted: On<MountedWindow>, mut commands: Commands) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let crate::window::Window::BattleDebug(_) = &mounted.0 {
        commands.trigger(Refresh)
    }
}

fn on_unmounted_window(unmounted: On<UnmountedWindow>, mut window: ResMut<DebugBattleWindow>) {
    // Store unmounted debug window to reuse it later when want to display it again
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let crate::window::Window::BattleDebug(window_) = &unmounted.0 {
        window.0 = Some(window_.clone())
    }
}
