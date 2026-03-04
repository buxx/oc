use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Home,
    Connecting,
    Downloading,
    InGame,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum InGameState {
    #[default]
    Battle,
    World,
}

#[derive(Debug, Resource, Default)]
pub struct Config(pub Option<oc_root::config::Config>);

#[derive(Debug, Resource, Default)]
pub struct Meta(pub Option<oc_world::meta::Meta>);

#[derive(Debug, Resource, Default)]
pub struct Window(pub Option<crate::window::Window>);
