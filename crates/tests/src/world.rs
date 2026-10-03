use bon::Builder;
use oc_geo::tile::WorldTileIndex;
use oc_individual::{Individual, IndividualIndex, squad::Squad};
use oc_mod::{Mod, nature::NatureIndex};
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{WorldConfig, battle::BattlePhase};
use oc_world::{
    World,
    meta::Meta,
    navmesh::{Walls, navmesh},
    spawn::SpawnZoneName,
    tile::Tile,
    visibility::Visibilities,
};
use rustc_hash::FxHashMap;

use crate::{squad::TestSquad, utils::workspace_path};

fn mod_() -> Mod {
    let path = workspace_path("mods/tests1");
    Mod::load(&path, None).unwrap()
}

#[derive(Debug, Builder)]
pub struct TestWorld {
    #[builder(default = mod_())]
    mod_: Mod,
    #[builder(default)]
    meta: Meta,
    #[builder(default = BattlePhase::Deployment)]
    phase: BattlePhase,
    #[builder(default)]
    side_a_spawns: Vec<SpawnZoneName>,
    #[builder(default)]
    side_b_spawns: Vec<SpawnZoneName>,
    tiles: Option<Vec<Tile>>,
    #[builder(default)]
    individuals: Vec<Individual>,
    squads: Option<Vec<Squad>>,
    #[builder(default)]
    projectiles: FxHashMap<ProjectileId, Projectile>,
}

impl TestWorld {
    pub fn make(self, w: &WorldConfig) -> World {
        let tiles = self.tiles.unwrap_or_else(|| {
            (0..w.tiles_count())
                .map(|i| {
                    let nature = NatureIndex(0);
                    let traversability = self.mod_.nature(nature).traversability.clone();
                    Tile::new(WorldTileIndex(i), nature, 0, traversability)
                })
                .collect()
        });
        let walls = tiles.as_walls(&self.mod_);
        let navmesh = navmesh(w, &walls);
        let squads = self.squads.unwrap_or_else(|| {
            self.individuals
                .iter()
                .enumerate()
                .map(|(i, individual)| {
                    TestSquad::builder()
                        .position(individual.position.into())
                        .members(vec![IndividualIndex(i as u64)])
                        .build()
                        .make()
                })
                .collect()
        });
        let individuals_count = self.individuals.len();

        World {
            w: w.clone(),
            mod_: self.mod_,
            meta: self.meta,
            tiles,
            navmesh,
            individuals: self.individuals,
            visibilities: Visibilities::empty(individuals_count),
            squads,
            projectiles: self.projectiles,
            phase: self.phase,
            side_a_spawns: self.side_a_spawns.clone(),
            side_b_spawns: self.side_b_spawns.clone(),
        }
    }
}
