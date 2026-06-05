use std::{marker::PhantomData, path::PathBuf};

use oc_individual::{Individual, squad::Squad};
use oc_mod::Mod;
use oc_projectile::Projectile;
use oc_root::WorldConfig;
use oc_world::{
    snapshot::{SaveError, Snapshot},
    tile::Tile,
};

pub mod individual;
pub mod projectile;
pub mod squad;
pub mod tile;

pub struct SnapshotBuilder<T, I, S, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    S: squad::SquadsGenerator,
    P: projectile::ProjectilesGenerator,
{
    tiles: T,
    individuals: I,
    squads: S,
    projectiles: P,
}

impl<T, I, S, P> SnapshotBuilder<T, I, S, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    S: squad::SquadsGenerator,
    P: projectile::ProjectilesGenerator,
{
    pub fn new(tiles: T, individuals: I, squads: S, projectiles: P) -> Self {
        Self {
            tiles,
            individuals,
            squads,
            projectiles,
        }
    }

    pub fn build(&self, w: WorldConfig, mod_: &Mod) -> Result<PathBuf, Error> {
        let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
        let tiles = self.tiles.tiles(&w, mod_);
        let individuals = self.individuals.individuals(&w, &tiles);
        let squads = self.squads.squads(&w, &individuals);
        let projectiles = self.projectiles.projectiles(&w, &tiles);

        let snapshot = Snapshot {
            w,
            tiles,
            individuals,
            squads,
            projectiles,
        };
        snapshot.save(&snapshot_path)?;

        Ok(snapshot_path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tempfile keep error: {0}")]
    Keep(#[from] tempfile::PersistError),
    #[error("Build error: {0}")]
    Save(#[from] SaveError),
}

pub struct EmptyGenerator<T>(PhantomData<T>);

impl<T> Default for EmptyGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EmptyGenerator<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl individual::IndividualsGenerator for EmptyGenerator<Individual> {
    fn individuals(&self, _: &WorldConfig, _: &Vec<Tile>) -> Vec<Individual> {
        vec![]
    }
}

impl squad::SquadsGenerator for EmptyGenerator<Squad> {
    fn squads(&self, _: &WorldConfig, _: &Vec<Individual>) -> Vec<Squad> {
        vec![]
    }
}

impl projectile::ProjectilesGenerator for EmptyGenerator<Projectile> {
    fn projectiles(&self, _: &WorldConfig, _: &Vec<Tile>) -> Vec<Projectile> {
        vec![]
    }
}

impl individual::IndividualsGenerator for Vec<Individual> {
    fn individuals(&self, _: &WorldConfig, _: &Vec<Tile>) -> Vec<Individual> {
        self.clone()
    }
}

impl projectile::ProjectilesGenerator for Vec<Projectile> {
    fn projectiles(&self, _: &WorldConfig, _: &Vec<Tile>) -> Vec<Projectile> {
        self.clone()
    }
}

impl squad::SquadsGenerator for Vec<Squad> {
    fn squads(&self, _: &WorldConfig, _: &Vec<Individual>) -> Vec<Squad> {
        self.clone()
    }
}

impl<T: Fn(&WorldConfig, &Vec<Tile>) -> Vec<Individual>> individual::IndividualsGenerator for T {
    fn individuals(&self, w: &WorldConfig, tiles: &Vec<Tile>) -> Vec<Individual> {
        self(w, tiles)
    }
}

impl<T: Fn(&WorldConfig, &Vec<Individual>) -> Vec<Squad>> squad::SquadsGenerator for T {
    fn squads(&self, w: &WorldConfig, individuals: &Vec<Individual>) -> Vec<Squad> {
        self(w, individuals)
    }
}

impl<T: Fn(&WorldConfig, &Vec<Tile>) -> Vec<Projectile>> projectile::ProjectilesGenerator for T {
    fn projectiles(&self, w: &WorldConfig, tiles: &Vec<Tile>) -> Vec<Projectile> {
        self(w, tiles)
    }
}
