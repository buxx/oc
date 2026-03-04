use oc_geo::{
    region::{Region, RegionXy},
    tile::TileXy,
};
use oc_physics::{Force, Physic};
use oc_projectile::{Projectile, ProjectileId, Update};

// TODO: lot of common code with individual
pub fn changes(
    id: &ProjectileId,
    projectile: &Projectile,
    position: &[f32; 2],
    forces: &Vec<Force>,
) -> Vec<Update> {
    let mut updates = vec![];

    if projectile.position() != position {
        updates.push(Update::UpdatePosition(*position));

        let tile: TileXy = position.clone().into();
        let region: RegionXy = tile.into();

        // FIXME BS NOW: do we need tile for a projectile ?
        if projectile.tile() != &tile {
            updates.push(Update::UpdateTile(tile));
        }

        if projectile.region() != &region {
            updates.push(Update::UpdateRegion(region));
        }
    }

    for force in projectile.forces() {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !projectile.forces().contains(force) {
            updates.push(Update::PushForce(force.clone()));
        }
    }

    tracing::trace!(name="physics-individual-updates", id=?id, projectile=?projectile, updates=?updates);

    updates
}
