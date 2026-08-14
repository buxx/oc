use bevy::prelude::*;
use oc_individual::order::Order;

use crate::sprites::{IntoIndividualSprite, IntoSprite, SpriteRect};

pub enum SquadOrderSprite {
    Idle,
    Move,
    MoveFast,
}

pub enum IndividualOrderSprite {
    Idle,
    Move,
    MoveFast,
}

impl IntoSprite<SquadOrderSprite> for Order {
    fn sprite(&self) -> SquadOrderSprite {
        match self {
            Order::Idle => SquadOrderSprite::Idle,
            Order::MoveTo(_) => SquadOrderSprite::Move,
            Order::MoveFastTo(_) => SquadOrderSprite::MoveFast,
        }
    }
}

impl SpriteRect for SquadOrderSprite {
    fn rect(&self) -> Rect {
        const START_X: f32 = 0.;
        const START_Y: f32 = 100.;
        const WIDTH: f32 = 11.;
        const HEIGHT: f32 = 11.;

        let i = match self {
            Self::Idle => -1, // Should never happen
            Self::Move => 0,
            Self::MoveFast => 1,
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}

impl IntoIndividualSprite<IndividualOrderSprite> for Order {
    fn individual_sprite(&self) -> IndividualOrderSprite {
        match self {
            Order::Idle => IndividualOrderSprite::Idle,
            Order::MoveTo(_) => IndividualOrderSprite::Move,
            Order::MoveFastTo(_) => IndividualOrderSprite::MoveFast,
        }
    }
}

impl SpriteRect for IndividualOrderSprite {
    fn rect(&self) -> Rect {
        const START_X: f32 = 22.;
        const START_Y: f32 = 100.;
        const WIDTH: f32 = 11.;
        const HEIGHT: f32 = 11.;

        let i = match self {
            Self::Idle => -1, // Should never happen
            Self::Move => 0,
            Self::MoveFast => 1,
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}
