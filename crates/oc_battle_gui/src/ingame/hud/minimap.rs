use bevy::prelude::*;

use oc_root::files;

use crate::{
    ingame::hud::{GAP, HUD_HEIGHT},
    network,
    states::GameConfig,
};

pub const MINIMAP_SIZE: f32 = HUD_HEIGHT - 2. * GAP;

#[derive(Component, Default, Clone)]
pub struct Minimap;

pub fn minimap() -> impl Scene {
    bsn! {
        Minimap
        Node {
            width: px(MINIMAP_SIZE),
            height: px(MINIMAP_SIZE),
            flex_shrink: 0.,
        }
        BackgroundColor(Color::BLACK)
    }
}

pub fn load_image(
    mut commands: Commands,
    slot: Query<Entity, (With<Minimap>, Without<ImageNode>)>,
    assets: Res<AssetServer>,
    g: Res<GameConfig>,
    network: Res<network::state::State>,
) {
    let Ok(entity) = slot.single() else { return };
    let (Some(g), Some(connect)) = (&g.0, &network.server) else {
        return;
    };
    let files = files::Files::new(g.mod_.canonical(), g.meta.canonical())
        .into_gui(g.static_.clone(), connect.clone().into());
    commands
        .entity(entity)
        .insert(ImageNode::new(assets.load(files.hud_minimap())));
}
