use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_physics::update::bevy::Region;
use oc_root::{Wcfg, WcfgInto, files, y::Y};

use crate::{
    entity::world::region::RegionBackground,
    ingame::draw::Z_REGION_BACKGROUND,
    network,
    states::{Meta, Mod, StaticSource},
};

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Debug, Event)]
pub struct ListeningRegion(pub WorldRegionIndex);

#[derive(Debug, Event)]
pub struct ForgottenRegion(pub WorldRegionIndex);

pub fn on_listening_region(
    region: On<ListeningRegion>,
    mut commands: Commands,
    w: Res<Wcfg>,
    assets: Res<AssetServer>,
    meta: Res<Meta>,
    mod_: Res<Mod>,
    static_: Res<StaticSource>,
    network: Res<network::state::State>,
) {
    let (Some(static_), Some(connect), Some(meta), Some(mod_), Some(w)) =
        (&static_.0, &network.server, &meta.0, &mod_.0, &w.0)
    else {
        return;
    };
    tracing::debug!("Spawn region background {:?}", region.0);

    let region_ = region.0;
    let mod_ = mod_.canonical();
    let world = meta.canonical();
    let files = files::Files::new(mod_, world).into_gui(static_.clone(), connect.clone().into());

    let region: RegionXy = region_.into_(w);

    let width = w.region_width_pixels as f32;
    let height = w.region_height_pixels as f32;
    let x = region.0.0 as f32 * width;
    let y = region.0.1 as f32 * height;
    let x = x + width / 2.;
    let y = y + height / 2.;
    let x = x;
    let y = y.to_gui_y(w);
    let i: WorldRegionIndex = region.into_(w);
    let background = files.region(i.0);

    tracing::trace!(name="spawn-region-background", region=?region, x=x, y=y, path=?background);
    commands.spawn((
        RegionBackground,
        Region(region_),
        Sprite::from_image(assets.load(background)),
        Transform {
            scale: Vec3::new(1.0, 1.0, 1.0),
            translation: Vec3::new(x, y, Z_REGION_BACKGROUND),
            ..default()
        },
    ));
}
