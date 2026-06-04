use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use oc_root::files::Files;

use crate::{
    ingame::GameConfigReceived, network, sprites::soldier::SoldierAnimations, states::GameConfig,
};

pub mod soldier;

#[derive(Debug, Default)]
pub struct Animations;

impl Plugin for Animations {
    fn build(&self, app: &mut App) {
        app.add_observer(on_game_config_received);
    }
}

fn on_game_config_received(
    _: On<GameConfigReceived>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    g: Res<GameConfig>,
    network: Res<network::state::State>,
    mut animations: ResMut<Assets<Animation>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let Some(g) = &g.0 else { return };
    let Some(connect) = network.server.clone() else {
        return;
    };

    let mod_ = g.mod_.canonical();
    let world = g.meta.canonical();
    let files = Files::new(mod_, world).into_gui(g.static_.clone(), connect.into());
    let sprites = files.sprites();

    let soldier = SoldierAnimations::init(&sprites, &assets, &mut animations, &mut atlas_layouts);

    commands.insert_resource(soldier);
}

pub trait IntoAnimation<A> {
    fn animation(&self, animations: &A) -> Handle<Animation>;
}
