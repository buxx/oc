use crate::physics::Meters;

#[derive(Debug, PartialEq)]
pub struct Explosion {
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub shrapnel_quantity: u32,
    pub penetration_force: f32,
}
