use bevy::prelude::*;
use oc_individual::order::Order;

use crate::sprites::{IntoSprite, SpriteRect};

pub enum SquadOrderSprite {
    Idle,
    Move,
}

impl IntoSprite<SquadOrderSprite> for Order {
    fn sprite(&self) -> SquadOrderSprite {
        match self {
            Order::Idle => SquadOrderSprite::Idle,
            Order::MoveTo(_) => SquadOrderSprite::Move,
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
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}
