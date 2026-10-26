use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use oc_root::Client;

use crate::{individual::death::die, physics::Processor, runner::update::Update};

impl<'x, E: Client> Processor<'x, E> {
    pub fn gunshot(&self, projectile: ProjectileId, i: IndividualIndex) -> Vec<Update> {
        let mut updates = vec![];
        let Some(_) = self.ctx.state.world().projectile(&projectile) else {
            return updates;
        };
        let world = self.ctx.state.world();
        let individual = world.individual(i);
        tracing::trace!(name="physics-collision-gunshot", projectile=?projectile, individual=?i);

        // TODO: Compute something according to projectile nature and body status
        if matches!(individual.status, oc_individual::Status::Operational) {
            #[cfg(feature = "tracker")]
            {
                self.ctx.tracker.take().individuals.push((
                    i,
                    oc_individual::Update::SetStatus(oc_individual::Status::Dead),
                ));
            }
            updates.extend(die(&world, self.ctx.state._mod(), i));
            updates.push(Update::RemoveProjectile(projectile));
        }

        updates
    }
}
