use bevy::prelude::*;

use crate::states::AppState;

pub mod actions;
pub mod minimap;
pub mod squad;

const HUD_HEIGHT: f32 = 200.;
const GAP: f32 = 4.;
const SCROLL_LINE_PX: f32 = 20.;
const HUD_BG: Color = Color::srgba(0., 0., 0., 0.95);

#[derive(Component, Default, Clone)]
pub struct Hud;

#[derive(Component, Default, Clone)]
pub struct Minimap;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud)
            .add_systems(
                Update,
                (
                    squad::spawn_squad_cells,
                    squad::scroll_squad_grid,
                    squad::update_squad_behaviors,
                    actions::update_fight_button,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn spawn_hud(mut commands: Commands, existing: Query<(), With<Hud>>) {
    if existing.is_empty() {
        commands.spawn_scene(hud());
    }
}

fn hud() -> impl Scene {
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
            (
                squad::SquadGrid
                Node {
                    flex_grow: 1.,
                    height: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: AlignContent::FlexStart,
                    row_gap: px(GAP),
                    column_gap: px(GAP),
                    overflow: Overflow::scroll_y(),
                }
            ),
            // Actions
            ({ actions::actions() }),
            // Minimap
            (
                Minimap
                Node {
                    width: px(minimap::MINIMAP_SIZE),
                    height: px(minimap::MINIMAP_SIZE),
                    flex_shrink: 0.,
                }
                BackgroundColor(Color::BLACK)
            ),
        ]
    }
}
