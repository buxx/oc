use crate::states::GuiFiles;
use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use oc_utils::let_some;

use crate::{ingame::GameConfigReceived, sprites::soldier::SoldierAnimations};

pub mod order;
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
    files: Res<GuiFiles>,
    mut animations: ResMut<Assets<Animation>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let_some!(files = &files.0, return);
    let sprites = files.sprites();

    let soldier = SoldierAnimations::init(&sprites, &assets, &mut animations, &mut atlas_layouts);

    commands.insert_resource(soldier);
}

pub trait IntoAnimation<A> {
    fn animation(&self, animations: &A) -> Handle<Animation>;
}

pub trait SpriteRect {
    fn rect(&self) -> Rect;
}

pub trait IntoSprite<T> {
    fn sprite(&self) -> T;
}

pub trait IntoIndividualSprite<T> {
    fn individual_sprite(&self) -> T;
}
