use bon::Builder;
use glam::Vec2;
use oc_geo::tile::WorldTileIndex;
use oc_individual::{Individual, IndividualIndex, squad::Squad};
use oc_mod::{Mod, nature::NatureIndex};
use oc_projectile::{Projectile, ProjectileId};
use oc_root::WorldConfig;
use oc_world::{World, meta::Meta, navmesh::Walls, navmesh::navmesh, tile::Tile};
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
            (0..w.tiles_count)
                .map(|i| {
                    let nature = NatureIndex(0);
                    let traversability = self.mod_.nature(nature).traversability.clone();
                    Tile::new(WorldTileIndex(i), nature, 0, traversability)
                })
                .collect()
        });
        let walls = tiles.as_walls(&self.mod_);
        let navmesh = navmesh(&w, &walls);
        let squads = self.squads.unwrap_or_else(|| {
            self.individuals
                .iter()
                .enumerate()
                .map(|(i, individual)| {
                    TestSquad::builder()
                        .position(Vec2::new(individual.position[0], individual.position[1]))
                        .members(vec![IndividualIndex(i as u64)])
                        .build()
                        .make()
                })
                .collect()
        });

        World {
            w: w.clone(),
            mod_: self.mod_,
            meta: self.meta,
            tiles: tiles,
            navmesh,
            individuals: self.individuals,
            squads,
            projectiles: self.projectiles,
        }
    }
}
