use std::fmt::Display;
use std::ops::Deref;

use derive_more::Constructor;
use oc_geo::Geo;
use oc_geo::UpdateGeo;
use oc_geo::region::Region;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::WorldTileIndex;
use oc_mod::Mod;
use oc_mod::ammunition::AmmunitionIndex;
use oc_mod::magazine::MagazineIndex;
use oc_mod::nature::Traversability;
use oc_mod::weapons::WeaponIndex;
use oc_physics::Force;
use oc_physics::IgnoreSide;
use oc_physics::Physic;
use oc_physics::UpdatePhysic;
use oc_physics::collision::Material;
use oc_physics::volume::Volume;
use oc_root::Suppress;
use oc_root::U8Progress;
use oc_root::WorldConfig;
use oc_root::geo::WorldVec3;
use oc_root::material::MaterialKind;
use oc_root::physics::Meters;
use oc_root::side::Side;
#[cfg(feature = "bevy")]
use oc_root::y::V;
use oc_utils::collections::WithIds;
use oc_utils::d2::Direction;
use rkyv::{Archive, Deserialize, Serialize};

use crate::behavior::Behavior;
use crate::behavior::Intent;
use crate::order::Order;

pub mod behavior;
pub mod network;
pub mod order;
pub mod squad;

pub const INDIVIDUAL_STAND_UP_VOLUME_WIDTH: Meters = Meters(0.8);
pub const INDIVIDUAL_STAND_UP_VOLUME_HEIGHT: Meters = Meters(0.8);
pub const INDIVIDUAL_STAND_UP_VOLUME_DEPTH: Meters = Meters(1.8);

// When maintain index of tile/individual, consider this shape to cover prone and rotation
pub const INDIVIDUAL_INDEXATION_SHAPE: Meters = Meters(2.0);

pub const INDIVIDUAL_PRONE_VOLUME_WIDTH: Meters = Meters(0.8);
pub const INDIVIDUAL_PRONE_VOLUME_HEIGHT: Meters = Meters(1.8);
pub const INDIVIDUAL_PRONE_VOLUME_DEPTH: Meters = Meters(0.8);

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Constructor,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndividualIndex(pub u64);

impl Display for IndividualIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Individual {
    pub side: Side,
    pub position: WorldVec3,
    pub tile: WorldTileIndex,
    pub region: WorldRegionIndex,
    pub orders: Vec<Order>,
    /// Behavior is the general behavior of individual, like defending a direction
    pub behavior: Behavior,
    /// Forces are physical forces applied to this individual
    pub forces: Vec<Force>,
    /// Status is "in game" status, like be able to act, or not (dead, incapacitated, etc)
    pub status: Status,
    /// Gesture is the physical gesture of individual, like walking to take its defending position
    pub gesture: Gesture,
    /// Intent is the intention of individual, like taking cover, to accomplish its defending behavior
    pub intent: Intent,
    pub suppress: Suppress,
    pub weapons: Weapons,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetForces(Vec<Force>),
    SetOrders(Vec<Order>),
    SetBehavior(Behavior),
    SetGesture(Gesture),
    SetStatus(Status),
    SetIntent(Intent),
    Accomplished,
    MoveStepAccomplished,
    SetWeapons(Weapons),
}

impl Region for Individual {
    fn region(&self) -> WorldRegionIndex {
        self.region
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.region = value;
    }
}

impl Deref for IndividualIndex {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for IndividualIndex {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<u64> for IndividualIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i32> for IndividualIndex {
    fn from(value: i32) -> Self {
        Self(value as u64)
    }
}

impl Individual {
    pub fn fresh(
        side: Side,
        position: WorldVec3,
        tile: WorldTileIndex,
        region: WorldRegionIndex,
    ) -> Self {
        Self::new(
            side,
            position,
            tile,
            region,
            vec![],
            Behavior::Idle(Direction::default()),
            vec![],
            Status::Operational,
            Gesture::body(BodyGesture::StandUp(Direction::default())),
            Intent::Idle(Direction::default()),
            Suppress::zero(),
            Weapons::default(),
        )
    }

    pub fn with_behavior(mut self, value: Behavior) -> Self {
        self.behavior = value;
        self
    }

    pub fn with_gesture(mut self, value: Gesture) -> Self {
        self.gesture = value;
        self
    }

    pub fn with_intent(mut self, value: Intent) -> Self {
        self.intent = value;
        self
    }

    pub fn with_weapons(mut self, value: Weapons) -> Self {
        self.weapons = value;
        self
    }

    pub fn tile(&self) -> WorldTileIndex {
        self.tile
    }

    pub fn region(&self) -> WorldRegionIndex {
        self.region
    }

    pub fn can_follow_orders(&self) -> bool {
        match self.status {
            Status::Operational => true,
            Status::Dead => false,
        }
    }

    pub fn can_lov(&self) -> bool {
        match self.status {
            Status::Operational => true,
            Status::Dead => false,
        }
    }

    /// True if compute lov on it (when dead or incapacitated, false)
    pub fn is_lov_target(&self) -> bool {
        match self.status {
            Status::Operational => true,
            Status::Dead => false,
        }
    }

    /// True if consider it as solid (when dead or incapacitated, false)
    pub fn is_solid(&self) -> bool {
        match self.status {
            Status::Operational => true,
            Status::Dead => false,
        }
    }

    pub fn is_always_visible(&self) -> bool {
        match self.status {
            Status::Operational => false,
            Status::Dead => true, // Dead are always displayed at screen
        }
    }

    pub fn xp_inaccuracy(&self, _w: &WorldConfig) -> f32 {
        // TODO
        0.0
    }

    pub fn suppress_inaccuracy(&self, w: &WorldConfig) -> f32 {
        self.suppress.normalize() * w.suppress_inaccuracy
    }
}

impl Physic for Individual {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        w: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        let direction = self.gesture.direction();
        let traversable = !self.is_solid();
        let cube = match self.gesture.body {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
                    height: INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
                    depth: INDIVIDUAL_STAND_UP_VOLUME_DEPTH.pixels(w),
                }
            }
            BodyGesture::Crawling(_direction) | BodyGesture::Prone(_direction) => Volume::Cube {
                x: ref_.x,
                y: ref_.y,
                z: ref_.z,
                width: INDIVIDUAL_PRONE_VOLUME_WIDTH.pixels(w),
                height: INDIVIDUAL_PRONE_VOLUME_HEIGHT.pixels(w),
                depth: INDIVIDUAL_PRONE_VOLUME_DEPTH.pixels(w),
            },
        };
        vec![(
            cube,
            Traversability {
                individual: true, // TODO: prevent individual collisions ? Will need enhance physic model ...
                projectile: traversable,
            },
            direction,
        )]
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn side(&self) -> Option<Side> {
        Some(self.side)
    }
}

impl UpdatePhysic for Individual {
    fn set_position(&mut self, value: WorldVec3) {
        self.position = value;
    }

    fn push_force(&mut self, value: Force) {
        self.forces.push(value)
    }

    fn remove_force(&mut self, value: &Force) {
        self.forces.retain(|f| f != value)
    }

    fn set_volumes(&self, _value: Vec<(Volume, Traversability)>) {
        // No update volume of an individual (for now ...)
    }
}

impl Geo for Individual {
    fn tile(&self) -> WorldTileIndex {
        self.tile
    }
}

impl UpdateGeo for Individual {
    fn set_tile(&mut self, value: WorldTileIndex) {
        self.tile = value;
    }
}

impl Material for Individual {
    fn kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Individual)
    }
}

impl<'a> WithIds<IndividualIndex, &'a Individual> for &'a [Individual] {
    fn with_ids(&self) -> Vec<(IndividualIndex, &'a Individual)> {
        self.iter()
            .enumerate()
            .map(|(i, individual)| (i.into(), individual))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Status {
    Operational,
    Dead,
}

impl Status {
    pub fn can_step(&self) -> bool {
        match self {
            Status::Operational => true,
            Status::Dead => false,
        }
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Gesture {
    pub body: BodyGesture,
    pub hands: HandsGesture,
}

impl Gesture {
    pub fn body(gesture: BodyGesture) -> Self {
        Self {
            body: gesture,
            hands: HandsGesture::Idle,
        }
    }

    pub fn with_hands(mut self, value: HandsGesture) -> Self {
        self.hands = value;
        self
    }

    pub fn inaccuracy(&self, w: &WorldConfig) -> f32 {
        match self.body {
            BodyGesture::StandUp(_) => w.standup_inaccuracy,
            BodyGesture::Walking(_) => w.walking_inaccuracy,
            BodyGesture::Running(_) => w.running_inaccuracy,
            BodyGesture::Crawling(_) => w.crawling_inaccuracy,
            BodyGesture::Prone(_) => w.prone_inaccuracy,
        }
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum BodyGesture {
    StandUp(Direction),
    Walking(Direction),
    Running(Direction),
    Crawling(Direction),
    Prone(Direction),
}

impl BodyGesture {
    /// Meters (from ground) to consider as weapon height
    pub fn weapon_z(&self) -> Meters {
        match self {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                Meters(1.5)
            }
            BodyGesture::Crawling(_) | BodyGesture::Prone(_) => Meters(0.35),
        }
    }

    /// Meters (from ground) to consider as target point
    pub fn target_z(&self) -> Meters {
        match self {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                Meters(0.75)
            }
            BodyGesture::Crawling(_) | BodyGesture::Prone(_) => Meters(0.30),
        }
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum HandsGesture {
    Idle,
    Reloading(U8Progress),
    Aiming(U8Progress),
}

// impl HandsGesture {
//     pub fn reloading(&self) -> Option<u8> {
//         match self {
//             HandsGesture::Idle => None,
//             HandsGesture::Reloading(progress) => Some(*progress),
//             HandsGesture::Aiming(_) => None,
//         }
//     }

//     pub fn aiming(&self) -> Option<u8> {
//         match self {
//             HandsGesture::Idle => None,
//             HandsGesture::Reloading(_) => None,
//             HandsGesture::Aiming(progress) => Some(*progress),
//         }
//     }
// }

impl Gesture {
    #[cfg(feature = "bevy")]
    pub fn rotation(&self, v: V) -> bevy::prelude::Quat {
        let angle = self.direction().angle(v);
        bevy::prelude::Quat::from_rotation_z(angle.0)
    }

    pub fn direction(&self) -> Direction {
        match self.body {
            BodyGesture::StandUp(direction)
            | BodyGesture::Walking(direction)
            | BodyGesture::Running(direction)
            | BodyGesture::Crawling(direction)
            | BodyGesture::Prone(direction) => direction.clone(),
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone, Default)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Weapons {
    pub primary: Option<Weapon>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Weapon {
    pub i: WeaponIndex,
    pub filled: Option<(MagazineIndex, AmmunitionIndex)>, // TODO: probably rethink arch for weapon without magazine
    pub filled_count: u16,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum WeaponKind {
    Primary,
}
