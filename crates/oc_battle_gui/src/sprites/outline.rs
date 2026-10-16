use bevy::prelude::*;

use crate::ingame::individual::Status;

const OPERATIONAL: Color = Color::srgba(0.1, 0.9, 0.1, 0.2);
const DEAD: Color = Color::srgba(0.9, 0.1, 0.1, 0.2);

#[derive(Debug, Component)]
pub struct IndividualOutline;

// Warning: AI generated
/// Keep outline on the same animation frame than its individual, and colored according to status
pub fn sync_individual_outlines(
    individuals: Query<(&Sprite, &Status), Without<IndividualOutline>>,
    mut outlines: Query<(&ChildOf, &mut Sprite), With<IndividualOutline>>,
) {
    for (parent, mut outline) in &mut outlines {
        let Ok((sprite, status)) = individuals.get(parent.parent()) else {
            continue;
        };

        let color = match status.0 {
            oc_individual::Status::Operational => OPERATIONAL,
            oc_individual::Status::Dead => DEAD,
        };
        if outline.color != color {
            outline.color = color;
        }

        let index = sprite.texture_atlas.as_ref().map(|atlas| atlas.index);
        if let (Some(index), Some(atlas)) = (index, outline.texture_atlas.as_mut())
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}
