use bevy::prelude::*;
use oc_individual::order::Order;

use crate::sprites::{IntoIndividualSprite, IntoSprite, SpriteRect};

pub enum SquadOrderSprite {
    Idle,
    Move,
    MoveFast,
    Defend,
    Hide,
}

pub enum IndividualOrderSprite {
    Idle,
    Move,
    MoveFast,
    Defend,
    Hide,
}

impl IntoSprite<SquadOrderSprite> for Order {
    fn sprite(&self) -> SquadOrderSprite {
        match self {
            Order::Idle => SquadOrderSprite::Idle,
            Order::MoveTo(_) => SquadOrderSprite::Move,
            Order::MoveFastTo(_) => SquadOrderSprite::MoveFast,
            Order::Defend(_) => SquadOrderSprite::Defend,
            Order::Hide(_) => SquadOrderSprite::Hide,
        }
    }
}

impl SpriteRect for SquadOrderSprite {
    fn rect(&self) -> Rect {
        const POSITION_START_X: f32 = 0.;
        const POSITION_START_Y: f32 = 100.;
        const POSITION_WIDTH: f32 = 11.;
        const POSITION_HEIGHT: f32 = 11.;

        const DIRECTION_START_X: f32 = 0.;
        const DIRECTION_START_Y: f32 = 145.;
        const DIRECTION_WIDTH: f32 = 50.;
        const DIRECTION_HEIGHT: f32 = 48.;

        match self {
            SquadOrderSprite::Idle => Rect::EMPTY, // Should never happen
            SquadOrderSprite::Move => {
                const INDEX: f32 = 0.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::MoveFast => {
                const INDEX: f32 = 1.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Defend => {
                const INDEX: f32 = 0.;
                let start_y = DIRECTION_START_Y + (INDEX * DIRECTION_HEIGHT);
                Rect::new(
                    DIRECTION_START_X,
                    start_y,
                    DIRECTION_START_X + DIRECTION_WIDTH,
                    start_y + DIRECTION_HEIGHT,
                )
            }
            SquadOrderSprite::Hide => {
                const INDEX: f32 = 1.;
                let start_y = DIRECTION_START_Y + (INDEX * DIRECTION_HEIGHT);
                Rect::new(
                    DIRECTION_START_X,
                    start_y,
                    DIRECTION_START_X + DIRECTION_WIDTH,
                    start_y + DIRECTION_HEIGHT,
                )
            }
        }
    }
}

impl IntoIndividualSprite<IndividualOrderSprite> for Order {
    fn individual_sprite(&self) -> IndividualOrderSprite {
        match self {
            Order::Idle => IndividualOrderSprite::Idle,
            Order::MoveTo(_) => IndividualOrderSprite::Move,
            Order::MoveFastTo(_) => IndividualOrderSprite::MoveFast,
            Order::Defend(_) => IndividualOrderSprite::Defend,
            Order::Hide(_) => IndividualOrderSprite::Hide,
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
            IndividualOrderSprite::Idle => return Rect::EMPTY, // Should never happen
            IndividualOrderSprite::Move => 0,
            IndividualOrderSprite::MoveFast => 1,
            IndividualOrderSprite::Defend => return Rect::EMPTY, // Should never happen
            IndividualOrderSprite::Hide => return Rect::EMPTY,   // Should never happen
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}
