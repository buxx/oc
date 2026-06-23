use bevy::prelude::*;

use update::on_update_squad;

#[cfg(feature = "debug")]
mod debug;
mod update;

pub struct SquadPlugin;

impl Plugin for SquadPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_squad);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::DebugPlugin);
        }
    }
}
