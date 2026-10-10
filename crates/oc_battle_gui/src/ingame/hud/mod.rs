use bevy::prelude::*;

use crate::{ingame::BattlePhase, states::AppState};

pub mod actions;
pub mod minimap;
pub mod squad;
pub mod squads;

const HUD_HEIGHT: f32 = 200.;
const GAP: f32 = 4.;
const SCROLL_LINE_PX: f32 = 20.;
const HUD_BG: Color = Color::srgba(0., 0., 0., 0.95);

#[derive(Component, Default, Clone)]
pub struct Hud;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud)
            .add_systems(
                Update,
                (
                    squads::spawn_squad_cells,
                    squads::scroll_squad_grid,
                    squads::update_squad_behaviors,
                    squad::update_rows,
                    minimap::load_image,
                    minimap::update_squad_markers,
                    minimap::update_viewport,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(BattlePhase::Fight), actions::update_fight_button)
            .add_observer(squad::spawn_rows);
    }
}

fn spawn_hud(
    mut commands: Commands,
    existing: Query<(), With<Hud>>,
    world: Res<crate::world::World>,
    phase: Res<State<BattlePhase>>,
) {
    if existing.is_empty() {
        commands.spawn_scene(hud(&world, &phase));
    }
}

fn hud(world: &crate::world::World, phase: &BattlePhase) -> impl Scene {
    bsn! {
        Hud
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            left: px(0),
            width: percent(100),
            height: px(HUD_HEIGHT),
            padding: UiRect::all(px(GAP)),
            column_gap: px(GAP),
        }
        BackgroundColor(HUD_BG)
        Children [
            // Squads
            ({ squads::squads() }),
            // Squad
            ({ squad::squad() }),
            // Actions
            ({ actions::actions(world, phase) }),
            // Minimap
            ({ minimap::minimap() }),
        ]
    }
}
