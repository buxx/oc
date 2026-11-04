use crate::physics::Meters;

#[derive(Debug, PartialEq)]
pub struct Explosion {
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub destruction_rayon: Meters,
    pub destruction_force: f32,
    pub shrapnel_quantity: u32,
    pub penetration_force: f32,
}
