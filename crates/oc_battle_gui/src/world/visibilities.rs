use bevy::prelude::*;
use oc_utils::let_some;

use crate::{
    entity::individual::{IndividualIndex, Side},
    world::{UpdateVisibilities, VisibilitiesUpdated},
};

#[derive(Debug, Clone, Deref, Default)]
pub struct Visibilities {
    visible: Vec<oc_individual::IndividualIndex>,
}

impl Visibilities {
    pub fn from_visibilities(
        visibilities: &Vec<(
            oc_individual::IndividualIndex,
            oc_individual::IndividualIndex,
            oc_world::visibility::Visibility,
        )>,
    ) -> Self {
        let visible = visibilities
            .iter()
            .filter_map(|(_, i2, v)| v.visible.then(|| *i2))
            .collect::<Vec<_>>();
        Self { visible }
    }
}

pub fn on_update_visibilities(
    event: On<UpdateVisibilities>,
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
) {
    world.visibilities = Visibilities::from_visibilities(&event.0);
    commands.trigger(VisibilitiesUpdated);
}

pub fn on_visibilities_updated(
    _: On<VisibilitiesUpdated>,
    mut query: Query<(&IndividualIndex, &mut Visibility, &Side)>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
) {
    let_some!(identity = &network.identity, return);

    for (i, mut visibility, side) in query.iter_mut() {
        if side.0 != identity.side {
            match world.visible(i.0) {
                true => *visibility = Visibility::Visible,
                false => *visibility = Visibility::Hidden,
            }
        }
    }
}
