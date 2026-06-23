use bevy::prelude::*;
use oc_individual::squad::SquadFormation;
use oc_physics::update::bevy::Position;
use oc_root::y::Y;
use oc_utils::bevy::EntityMapping;

use crate::{
    entity::individual::IndividualIndex,
    ingame::{
        camera::{self, debug::individual::ShowFormationPositions},
        individual::Gesture,
    },
    states::{AppState, GameConfig, InGameState},
    world,
};

const FORMATION_POSITON_COLOR: Color = Color::srgba(0.5, 1.0, 0.0, 0.2);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FormationPositionsGizmos>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                draw_formations
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct FormationPositionsGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-squad-debug-setup-formations-gizmos");
    let (gizmos, _) = config.config_mut::<FormationPositionsGizmos>();
    gizmos.line.width = 2.0;
}

pub fn draw_formations(
    g: Res<GameConfig>,
    show: Res<ShowFormationPositions>,
    mut gizmos: Gizmos<FormationPositionsGizmos>,
    state: Res<camera::State>,
    world: Res<world::World>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    individual: Query<(&Position, &Gesture), With<IndividualIndex>>,
) {
    if !show.0 {
        return;
    }
    let Some(g) = &g.0 else {
        return;
    };
    let Some(regions) = &state.regions else {
        return;
    };

    tracing::trace!(name = "ingame-squad-debug-formations-draw", regions=?regions);
    for region in regions {
        let Some(squads) = world.squads.get(&region.0) else {
            return;
        };

        for (i, squad) in squads {
            let leader = squad.leader();
            let count = squad.members.len(); // TODO: active members (compute must be cached in Squad)
            let Some(leader) = individuals.get(&leader) else {
                continue;
            };
            let Ok((position, gesture)) = individual.get(*leader) else {
                continue;
            };
            let position = Vec2::new(position.0[0], position.0[1].to_gui_y(&g.w));
            let angle = gesture.0.direction().angle();

            // FXME BS NOW: squad formation from squad instead hardcoded
            let positions = SquadFormation::Line.positions(&g.w, position, angle, count);
            for position in positions {
                tracing::trace!(name = "ingame-squad-debug-formations-draw-point", i=?i, position=?position);
                gizmos.circle_2d(position, 1., FORMATION_POSITON_COLOR);
            }
        }
    }
}
