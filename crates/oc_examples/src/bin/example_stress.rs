#![allow(clippy::unwrap_used)]
use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use oc_battle_gui::{ingame::GameConfigReceived, network::output::ToServerEvent};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_individual::{Individual, IndividualIndex, order::Order, squad::Squad};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::Side,
};
use oc_world::{meta::Meta, support::SupportType};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

// WARNING: AI generated module

const MOD: &str = "mods/std1";
const MAP: &str = "examples/world1";
const SQUADS_BY_SIDE: usize = 200;
const INDIVIDUALS_BY_SQUAD: usize = 5;
/// Squads grid of one side
const GRID_COLUMNS: usize = 10;
const GRID_ROWS: usize = SQUADS_BY_SIDE / GRID_COLUMNS;
/// Distance between individuals of a same squad (pixels)
const MEMBERS_GAP: f32 = 6.;
const MARGIN: f32 = 30.;
/// Machine gun given to every individual, to stress projectiles physics
const WEAPON: &str = "Mg34";
/// Spare magazines by individual
const SPARE_MAGAZINES: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    #[cfg(feature = "debug")]
    WorldConfig::set_immortality(true);
    #[cfg(not(feature = "debug"))]
    tracing::warn!("`debug` feature disabled: no immortality, only side A requests bombardments");

    let mod_path = PathBuf::from(MOD);
    let mod_ = Mod::load(&mod_path, None)?;
    let meta = Meta::from_file(&PathBuf::from(MAP).join("meta.toml"))?;
    let map = oc_world::reader::MapReader::new(&PathBuf::from(MAP));
    let map = map.context(format!("Read map {MAP}"))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));

    let (individuals, squads) = battalions(&w, &mod_);
    let snapshot = SnapshotBuilder::new(
        map,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w.clone(), &mod_)?;

    let bombard = Bombard::new(&w, &mod_);
    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(mod_path)
        .install(Box::new(move |app| install(app, bombard.clone())))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}

fn squad_position(w: &WorldConfig, side: Side, j: usize) -> WorldVec2 {
    let width = w.world_width_pixels() as f32 - 2. * MARGIN;
    let height = w.world_height_pixels() as f32 - 2. * MARGIN;
    let (cell_w, cell_h) = (width / GRID_COLUMNS as f32, height / GRID_ROWS as f32);
    let shift = match side {
        Side::A => 0.25,
        Side::B => 0.75,
    };
    let (column, row) = (j % GRID_COLUMNS, j / GRID_COLUMNS);
    WorldVec2::new(
        MARGIN + (column as f32 + shift) * cell_w,
        MARGIN + (row as f32 + shift) * cell_h,
    )
}

fn battalions(w: &WorldConfig, mod_: &Mod) -> (Vec<Individual>, Vec<Squad>) {
    let (width, height) = (
        w.world_width_pixels() as f32,
        w.world_height_pixels() as f32,
    );
    let spare_magazines = vec![
        mod_.weapons
            .iter()
            .find(|w| w.name() == WEAPON)
            .unwrap()
            .magazines()
            .first()
            .unwrap()
            .index();
        SPARE_MAGAZINES
    ];
    let mut individuals = vec![];
    let mut squads = vec![];

    for side in [Side::A, Side::B] {
        for j in 0..SQUADS_BY_SIDE {
            let position = squad_position(w, side, j);
            let first = individuals.len();
            for k in 0..INDIVIDUALS_BY_SQUAD {
                individuals.push(
                    TestIndividual::builder()
                        .side(side)
                        .position(WorldVec3::new(
                            position.x + k as f32 * MEMBERS_GAP,
                            position.y,
                            0.,
                        ))
                        .weapons(
                            TestWeapons::builder()
                                .primary(TestWeapon::filled(mod_, WEAPON).make())
                                .build()
                                .make(),
                        )
                        .magazines(spare_magazines.clone())
                        .build()
                        .make(w),
                );
            }

            // Odd squads move to the opposite position on the map
            let orders = match j % 2 {
                1 => vec![Order::MoveTo(WorldVec2::new(
                    width - position.x,
                    height - position.y,
                ))],
                _ => vec![],
            };
            squads.push(
                TestSquad::builder()
                    .side(side)
                    .position(position)
                    .members(
                        (first..first + INDIVIDUALS_BY_SQUAD)
                            .map(|i| IndividualIndex(i as u64))
                            .collect(),
                    )
                    .orders(orders)
                    .build()
                    .make(),
            );
        }
    }

    (individuals, squads)
}

#[derive(Resource, Clone)]
struct Bombard {
    width: f32,
    height: f32,
    period: f32,
    next: [f32; 2],
    started: bool,
    rng: u32,
}

impl Bombard {
    fn new(w: &WorldConfig, mod_: &Mod) -> Self {
        let support = &mod_.support;
        let shells = support.naval_bombard_shells_by_serie;
        let serie = support.naval_bombard_shell_interval() * shells
            + support.naval_bombard_serie_interval();
        // Last explosion of the last shell of the last serie
        let last = support.naval_bombard_prepare()
            + serie * support.naval_bombard_series.saturating_sub(1)
            + support.naval_bombard_shell_interval() * shells.saturating_sub(1);

        Self {
            width: w.world_width_pixels() as f32,
            height: w.world_height_pixels() as f32,
            period: last.as_secs_f32() / w.speed(),
            next: [0.; 2],
            started: false,
            rng: 0x9E37_79B9,
        }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

fn install(app: &mut App, bombard: Bombard) {
    app.insert_resource(bombard)
        .add_observer(|_: On<GameConfigReceived>, mut bombard: ResMut<Bombard>| {
            tracing::info!("Game config received, start bombardments");
            bombard.started = true;
        })
        .add_systems(Update, request_bombards);
}

fn request_bombards(mut bombard: ResMut<Bombard>, time: Res<Time<Real>>, mut commands: Commands) {
    if !bombard.started {
        return;
    }

    let now = time.elapsed_secs();
    for (i, side) in [Side::A, Side::B].into_iter().enumerate() {
        if now < bombard.next[i] {
            continue;
        }
        bombard.next[i] = now + bombard.period;

        let position = WorldVec2::new(
            bombard.random() * bombard.width,
            bombard.random() * bombard.height,
        );
        #[cfg(not(feature = "debug"))]
        if side == Side::B {
            continue;
        }
        tracing::info!(?side, ?position, "Request naval bombard");
        #[cfg(feature = "debug")]
        commands.trigger(ToServerEvent(ToServer::DebugRequestSupport(
            side,
            SupportType::NavalBombard,
            position,
        )));
        #[cfg(not(feature = "debug"))]
        commands.trigger(ToServerEvent(ToServer::RequestSupport(
            SupportType::NavalBombard,
            position,
        )));
    }
}
