use oc_geo::{region::RegionXy, tile::TileXy};
use oc_individual::{Individual, IndividualIndex, Update};
use oc_physics::{Force, Physic};

pub fn changes(
    i: IndividualIndex,
    individual: &Individual,
    position: &[f32; 2],
    forces: &Vec<Force>,
) -> Vec<Update> {
    let mut updates = vec![];

    if individual.position() != position {
        updates.push(Update::UpdatePosition(*position));

        let tile: TileXy = position.clone().into();
        let region: RegionXy = tile.into();

        if individual.tile() != &tile {
            updates.push(Update::UpdateTile(tile));
        }

        if individual.region() != &region {
            updates.push(Update::UpdateRegion(region));
        }
    }

    for force in individual.forces() {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !individual.forces().contains(force) {
            updates.push(Update::PushForce(force.clone()));
        }
    }

    tracing::trace!(name="physics-individual-updates", i=?i, updates=?updates);

    updates
}
