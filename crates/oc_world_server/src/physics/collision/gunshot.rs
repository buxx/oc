use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use oc_root::Client;

use crate::{physics::Processor, runner::update::Update};

impl<'x, E: Client> Processor<'x, E> {
    pub fn gunshot(&self, projectile: ProjectileId, i: IndividualIndex) -> Vec<Update> {
        let mut updates = vec![];
        let Some(_) = self.ctx.state.world().projectile(&projectile) else {
            return updates;
        };
        let world = self.ctx.state.world();
        let individual = world.individual(i);
        tracing::trace!(name="physics-collision-gunshot", projectile=?projectile, individual=?i);

        // TODO: probably dedicated code instead simple like this
        match individual.status {
            oc_individual::Status::Dead => {}
            // TODO: Compute something according to projectile nature and body status
            oc_individual::Status::Operational => {
                let dead = oc_individual::Status::Dead;
                let status = oc_individual::Update::SetStatus(dead);
                updates.push(Update::UpdateIndividual(i, status));
                updates.push(Update::RemoveProjectile(projectile));
            }
        }

        updates
    }
}
