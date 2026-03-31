use std::{marker::PhantomData, path::PathBuf};

use oc_individual::Individual;
use oc_projectile::Projectile;
use oc_world::{
    snapshot::{SaveError, Snapshot},
    tile::Tile,
};

pub mod individual;
pub mod projectile;
pub mod tile;

pub struct SnapshotBuilder<T, I, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    P: projectile::ProjectilesGenerator,
{
    tiles: T,
    individuals: I,
    projectiles: P,
}

impl<T, I, P> SnapshotBuilder<T, I, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    P: projectile::ProjectilesGenerator,
{
    pub fn new(tiles: T, individuals: I, projectiles: P) -> Self {
        Self {
            tiles,
            individuals,
            projectiles,
        }
    }

    pub fn build(&self) -> Result<PathBuf, Error> {
        let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
        let tiles = self.tiles.tiles();
        let individuals = self.individuals.individuals(&tiles);
        let projectiles = self.projectiles.projectiles();

        let snapshot = Snapshot {
            tiles,
            individuals,
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

impl<T> EmptyGenerator<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl individual::IndividualsGenerator for EmptyGenerator<Individual> {
    fn individuals(&self, _: &Vec<Tile>) -> Vec<Individual> {
        vec![]
    }
}

impl projectile::ProjectilesGenerator for EmptyGenerator<Projectile> {
    fn projectiles(&self) -> Vec<Projectile> {
        vec![]
    }
}

impl individual::IndividualsGenerator for Vec<Individual> {
    fn individuals(&self, _: &Vec<Tile>) -> Vec<Individual> {
        self.clone()
    }
}

impl projectile::ProjectilesGenerator for Vec<Projectile> {
    fn projectiles(&self) -> Vec<Projectile> {
        self.clone()
    }
}

impl<T: Fn(&Vec<Tile>) -> Vec<Individual>> individual::IndividualsGenerator for T {
    fn individuals(&self, tiles: &Vec<Tile>) -> Vec<Individual> {
        self(tiles)
    }
}
