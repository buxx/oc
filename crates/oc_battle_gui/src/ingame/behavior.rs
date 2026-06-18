use bevy::prelude::*;
use oc_physics::update::bevy::Position;
use oc_root::y::Y;

use crate::{
    entity::individual::{IndividualIndex, Intent},
    ingame::draw,
    states::{GameConfig, InGameState},
};

const PATH_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

pub struct BehaviorPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PathGizmos;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PathGizmos>()
            .add_systems(Startup, setup)
            .add_systems(Update, draw_paths.run_if(in_state(InGameState::Battle)));
    }
}

fn setup(mut config: ResMut<GizmoConfigStore>) {
    let (gizmos, _) = config.config_mut::<PathGizmos>();
    gizmos.line.width = 1.0;
    gizmos.line.style = GizmoLineStyle::Dotted;
}

fn draw_paths(
    g: Res<GameConfig>,
    intents: Query<(&Intent, &Position), With<IndividualIndex>>,
    mut gizmos: Gizmos<PathGizmos>,
) {
    let Some(g) = &g.0 else {
        return;
    };

    for (intent, position) in intents {
        match &intent.0 {
            oc_individual::behavior::Intent::Idle(_) => {}
            oc_individual::behavior::Intent::MoveTo(_, path) => {
                let mut previous: [f32; 2] = [position.0[0], position.0[1]];
                for point in path.iter() {
                    let start = Vec3::new(previous[0], previous[1].to_gui_y(&g.w), draw::Z_PATH);
                    let stop = Vec3::new(point[0], point[1].to_gui_y(&g.w), draw::Z_PATH);
                    gizmos.line(start, stop, PATH_COLOR);

                    previous = [point[0], point[1]];
                }
            }
        }
    }
}
