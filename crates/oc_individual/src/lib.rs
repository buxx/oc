use derive_more::Constructor;
use oc_utils::d2::Xy;

use crate::behavior::Behavior;

pub mod behavior;

#[derive(Debug, Constructor)]
pub struct Individual {
    pub xy: Xy,
    pub behavior: Behavior,
}
