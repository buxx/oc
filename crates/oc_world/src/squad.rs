use oc_individual::squad::SquadFormation;

use crate::World;

pub struct SquadPositions(Vec<f32>);

impl SquadPositions {
    pub fn compute(
        world: &World,
        leader_position: &[f32; 2],
        formation: SquadFormation,
        members_count: usize,
    ) -> Self {
        todo!()
    }
}
