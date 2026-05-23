use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use oc_battle_gui::ingame::lov::{Lov, UpdateLovFor};
use oc_battle_gui::ingame::{ModReceived, WcfgReceived};
use oc_battle_gui::states::Mod;
use oc_battle_gui::world::InsertedTiles;
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_root::Wcfg;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::meta::Meta;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = oc_mod::Mod::load(&PathBuf::from("mods/std1"), None)?;
    let meta = Meta::from_file(&PathBuf::from("examples/height/meta.toml"))?;
    let map_ = PathBuf::from("examples/height");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let snapshot = SnapshotBuilder::new(map, vec![], vec![]).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from("examples/height"))
        .mod_(PathBuf::from("mods/std1"))
        .install(Box::new(setup))
        .snapshot(snapshot);

    let _ = example.build().run()?;

    Ok(())
}

#[derive(Debug, Event)]
struct SetupLovs;

fn setup(app: &mut bevy::app::App) {
    app.add_observer(on_wcfg_received)
        .add_observer(on_mod_received)
        .add_observer(on_inserted_tiles)
        .add_observer(on_setup_lovs);
}

// TODO: This is a tricky way to know when on_update_lov_for is ready to receive
// find a way to trigger when all these config are received, or send all in same time
fn on_wcfg_received(
    _: On<WcfgReceived>,
    mod_: Res<Mod>,
    mut commands: Commands,
    world: Res<oc_battle_gui::world::World>,
) {
    if mod_.0.is_some() && !world.tiles.is_empty() {
        commands.trigger(SetupLovs);
    }
}

// TODO: This is a tricky way to know when on_update_lov_for is ready to receive
// find a way to trigger when all these config are received, or send all in same time
fn on_mod_received(
    _: On<ModReceived>,
    w: Res<Wcfg>,
    mut commands: Commands,
    world: Res<oc_battle_gui::world::World>,
) {
    if w.0.is_some() && !world.tiles.is_empty() {
        commands.trigger(SetupLovs);
    }
}

// TODO: This is a tricky way to know when on_update_lov_for is ready to receive
// find a way to trigger when all these config are received, or send all in same time
fn on_inserted_tiles(_: On<InsertedTiles>, w: Res<Wcfg>, mod_: Res<Mod>, mut commands: Commands) {
    if w.0.is_some() && mod_.0.is_some() {
        commands.trigger(SetupLovs);
    }
}

fn on_setup_lovs(_: On<SetupLovs>, w: Res<Wcfg>, mut commands: Commands) {
    let lovs = vec![
        // dans la plaine, aucun obstacle
        (Vec3::new(65., 50., 12.), Vec2::new(183., 50.)),
        // plaine, mur de brique
        (Vec3::new(71., 124., 7.5), Vec2::new(181., 123.)),
        // plaine grande montagne
        (Vec3::new(70., 172., 7.), Vec2::new(183., 172.)),
        // plaine petite montagne
        (Vec3::new(71., 227., 7.), Vec2::new(185., 227.)),
        // plaine herbe hautes
        (Vec3::new(65., 77., 12.), Vec2::new(180., 76.)),
        // plaine sous bois
        (Vec3::new(65., 22., 12.), Vec2::new(184., 22.)),
        // semi colline par dessus moyenne montagne
        (Vec3::new(29., 242., 52.), Vec2::new(216., 241.)),
        // haute colline par dessus moyenne montagne
        (Vec3::new(11., 207., 57.), Vec2::new(241., 207.)),
        // semi colline haute montagne
        (Vec3::new(40., 188., 37.), Vec2::new(214., 186.)),
        // haute colline par dessus haute montagne
        (Vec3::new(3., 157., 57.), Vec2::new(246., 157.)),
        // semi colline par dessus mur brique
        (Vec3::new(30., 131., 47.), Vec2::new(222., 131.)),
        // semi colline par dessus herbe haute
        (Vec3::new(13., 71., 57.), Vec2::new(241., 71.)),
        // semi colline par dessus sous bois
        (Vec3::new(31., 30., 47.), Vec2::new(233., 30.)),
    ];

    for (start, end) in lovs {
        let lov = Lov {
            start,
            stop: start,
            stop_plus_z: Meters(0.),
            sections: vec![],
        };
        let entity = commands.spawn(lov);
        let update = UpdateLovFor(entity.id(), end);
        commands.trigger(update);
    }
}
