use std::fmt::Display;

use derive_more::Constructor;

use crate::window::debug::physics::PhysicsRepr;

#[derive(Debug, Clone, Constructor)]
pub struct Subject<I: Display> {
    pub i: I,
    pub physics: PhysicsRepr,
}
