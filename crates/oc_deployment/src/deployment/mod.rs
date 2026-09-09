use std::path::PathBuf;

use anyhow::Context;
use oc_geo::tile::WorldTileIndex;
use oc_individual::{Individual, IndividualIndex, Weapon, squad::Squad};
use oc_mod::Mod;
use oc_root::{WcfgFrom, WorldConfig, geo::WorldVec2, side::Side};
use oc_world::{snapshot::Snapshot, tile::Tile};
use serde::{Deserialize, Serialize};

use crate::deployment::place::Placer;

pub mod individual;
pub mod magazine;
pub mod place;
pub mod squad;
pub mod weapon;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Deployments {
    side_a: Deployment,
    side_b: Deployment,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Deployment {
    pub squads: Vec<squad::Squad>,
}

impl Deployment {
    pub fn from_file(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let raw = std::fs::read_to_string(path).context(format!("Read file {}", path.display()))?;
        let deployment = serde_yaml::from_str(&raw);
        let deployment = deployment.context(format!("Deserialize from file {}", path.display()))?;
        Ok(deployment)
    }
}

impl Deployments {
    pub fn from_files(side_a: &PathBuf, side_b: &PathBuf) -> Result<Self, anyhow::Error> {
        Ok(Self {
            side_a: Deployment::from_file(side_a)?,
            side_b: Deployment::from_file(side_b)?,
        })
    }

    pub fn mobilize(
        &self,
        w: &WorldConfig,
        mod_: &Mod,
        snapshot: &mut Snapshot,
        placer: &Placer,
        tiles: &Vec<Tile>,
    ) -> Result<(), anyhow::Error> {
        snapshot.squads.clear();
        snapshot.individuals.clear();
        let mut i: u64 = 0;

        let mut fill_with = |deployment: &Deployment, side: Side| -> Result<(), anyhow::Error> {
            for squad in &deployment.squads {
                fill_squad(w, mod_, snapshot, placer, &mut i, side, squad, tiles)?;
            }

            Ok(())
        };

        fill_with(&self.side_a, Side::A)?;
        fill_with(&self.side_b, Side::B)?;

        Ok(())
    }
}

fn fill_squad(
    w: &WorldConfig,
    mod_: &Mod,
    snapshot: &mut Snapshot,
    placer: &Placer,
    i: &mut u64,
    side: Side,
    squad: &squad::Squad,
    tiles: &Vec<Tile>,
) -> Result<(), anyhow::Error> {
    let mut individuals = vec![];
    let position = placer.place(squad.uuid);

    for individual_ in &squad.individuals {
        let individual = fill_individual(w, mod_, i, side, individual_, position, tiles)?;
        individuals.push(IndividualIndex(*i));
        snapshot.individuals.push(individual);
        *i += 1;
    }
    snapshot
        .squads
        .push(Squad::fresh(side, individuals, position));
    Ok(())
}

fn fill_individual(
    w: &WorldConfig,
    mod_: &Mod,
    individual_index: &mut u64,
    side: Side,
    individual_: &individual::Individual,
    squad_position: WorldVec2,
    tiles: &Vec<Tile>,
) -> Result<Individual, anyhow::Error> {
    // FIXME BS NOW: z; and according to formation (and according to best opacity tile)
    let tile = WorldTileIndex::from_(squad_position, w);
    let tile = &tiles[tile.0 as usize];
    let z = tile.z_pixels(w);
    let position = squad_position.extend(z);
    let mut individual = Individual::fresh(w, side, position);

    if let Some(weapon) = &individual_.weapons.main {
        let weapon = mod_.weapon_from_name(weapon)?;
        let (filled, filled_count) = fill_weapon(mod_, individual_, weapon).context(format!(
            "Fill weapon for individual {individual_index} and mod {}",
            mod_.name()
        ))?;
        individual.weapons.primary = Some(Weapon {
            i: weapon.index(),
            filled,
            filled_count,
        });
    }

    for magazine in &individual_.magazines {
        let magazine = mod_.magazine_from_name(&magazine.name).context(format!(
            "Fill magazine for individual {individual_index} and magazine {} and mod {}",
            magazine.name,
            mod_.name()
        ))?;

        individual.magazines.push(magazine.index());
    }

    Ok(individual)
}

// Search from deployment individual magazines if one match with weapon compatible magazines
fn fill_weapon(
    mod_: &Mod,
    deployment_individual: &individual::Individual,
    weapon: &oc_mod::weapons::IndexedWeapon,
) -> Result<
    (
        Option<(
            oc_mod::magazine::MagazineIndex,
            oc_mod::ammunition::AmmunitionIndex,
        )>,
        u16,
    ),
    anyhow::Error,
> {
    for magazine in &deployment_individual.magazines {
        // If it is an accepted magazine by this weapon
        if weapon.magazines().iter().any(|m| m.name() == magazine.name) {
            // Read real magazine from mod
            let magazine = mod_.magazine_from_name(&magazine.name)?;

            // This magazine must accept ammunition model which is compatible with weapon
            let magazine_ammunitions = mod_.ammunitions_from_model_names(magazine.accept());
            if let Some(ammunition) = weapon
                .ammunitions()
                .iter()
                .find(|a| magazine_ammunitions.contains(&a))
            {
                return Ok((
                    Some((magazine.index(), ammunition.index())),
                    magazine.capacity(),
                ));
            }
        }
    }

    return Ok((None, 0));
}
