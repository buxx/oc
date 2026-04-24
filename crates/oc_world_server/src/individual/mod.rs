use derive_more::Constructor;
use oc_individual::IndividualIndex;
use oc_root::Client;

use crate::{individual::move_::Move, utils::context::Context};

mod move_;
pub mod physics;
pub mod update;

#[derive(Constructor)]
pub struct Processor<'a, E: Client> {
    ctx: &'a Context<E>,
    i: IndividualIndex,
}

impl<'a, E: Client> Processor<'a, E> {
    pub fn step(self) {
        let mut updates = vec![];

        updates.extend(Move::from(&self).read());

        updates.into_iter().for_each(|update| {
            update::write(self.ctx, update, self.i);
        });
    }
}
