use std::path::PathBuf;

use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use derive_more::Constructor;
use oc_root::side::Side;

use crate::sprites::IntoAnimation;

const COLUMNS: usize = 8;
const ROWS: usize = 12;
const IMAGE_WIDTH: u32 = 512;
const IMAGE_HEIGHT: u32 = 768;
const SIDE_B_START_ROW: usize = 6;

// soldiers.png
// TODO: use generative macro to write less code (animations creations and functions to get sprite + animation)
#[derive(Debug, Resource)]
pub struct SoldierAnimations {
    sprite: Sprite,
    side_a_stand_up: Handle<Animation>,
    side_a_walking: Handle<Animation>,
    side_a_running: Handle<Animation>,
    side_a_crawling: Handle<Animation>,
    side_a_prone: Handle<Animation>,
    side_a_dead_prone: Handle<Animation>,
    #[allow(unused)]
    side_a_hurt_prone: Handle<Animation>,
    side_b_stand_up: Handle<Animation>,
    side_b_walking: Handle<Animation>,
    side_b_running: Handle<Animation>,
    side_b_crawling: Handle<Animation>,
    side_b_prone: Handle<Animation>,
    side_b_dead_prone: Handle<Animation>,
    #[allow(unused)]
    side_b_hurt_prone: Handle<Animation>,
}

impl SoldierAnimations {
    pub fn init(
        sprites: &PathBuf,
        assets: &AssetServer,
        animations: &mut Assets<Animation>,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let image = assets.load(sprites.join("soldiers.png"));
        let spritesheet = Spritesheet::new(&image, COLUMNS, ROWS);

        let side_a_stand_up = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 0, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_a_walking = spritesheet
            .create_animation()
            .add_row(1)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_a_running = spritesheet
            .create_animation()
            .add_row(1)
            .set_duration(AnimationDuration::PerFrame(250))
            .build();
        let side_a_crawling = spritesheet
            .create_animation()
            .add_row(2)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_a_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 3, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_a_dead_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_a_hurt_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 5, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();

        let side_a_stand_up = animations.add(side_a_stand_up);
        let side_a_walking = animations.add(side_a_walking);
        let side_a_running = animations.add(side_a_running);
        let side_a_crawling = animations.add(side_a_crawling);
        let side_a_prone = animations.add(side_a_prone);
        let side_a_dead_prone = animations.add(side_a_dead_prone);
        let side_a_hurt_prone = animations.add(side_a_hurt_prone);

        let side_b_stand_up = spritesheet
            .create_animation()
            .add_horizontal_strip(0, SIDE_B_START_ROW, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_walking = spritesheet
            .create_animation()
            .add_row(SIDE_B_START_ROW + 1)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_b_running = spritesheet
            .create_animation()
            .add_row(SIDE_B_START_ROW + 1)
            .set_duration(AnimationDuration::PerFrame(250))
            .build();
        let side_b_crawling = spritesheet
            .create_animation()
            .add_row(SIDE_B_START_ROW + 2)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_b_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, SIDE_B_START_ROW + 3, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_dead_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, SIDE_B_START_ROW + 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_hurt_prone = spritesheet
            .create_animation()
            .add_horizontal_strip(0, SIDE_B_START_ROW + 5, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();

        let side_b_stand_up = animations.add(side_b_stand_up);
        let side_b_walking = animations.add(side_b_walking);
        let side_b_running = animations.add(side_b_running);
        let side_b_crawling = animations.add(side_b_crawling);
        let side_b_prone = animations.add(side_b_prone);
        let side_b_dead_prone = animations.add(side_b_dead_prone);
        let side_b_hurt_prone = animations.add(side_b_hurt_prone);

        let sprite = spritesheet
            .with_size_hint(IMAGE_WIDTH, IMAGE_HEIGHT)
            .sprite(atlas_layouts);

        Self {
            sprite,
            side_a_stand_up,
            side_a_walking,
            side_a_running,
            side_a_crawling,
            side_a_prone,
            side_a_dead_prone,
            side_a_hurt_prone,
            side_b_stand_up,
            side_b_walking,
            side_b_running,
            side_b_crawling,
            side_b_prone,
            side_b_dead_prone,
            side_b_hurt_prone,
        }
    }

    pub fn sprite(&self) -> Sprite {
        self.sprite.clone()
    }

    pub fn stand_up(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_stand_up.clone(),
            Side::B => self.side_b_stand_up.clone(),
        }
    }

    pub fn walking(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_walking.clone(),
            Side::B => self.side_b_walking.clone(),
        }
    }

    pub fn running(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_running.clone(),
            Side::B => self.side_b_running.clone(),
        }
    }

    pub fn crawling(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_crawling.clone(),
            Side::B => self.side_b_crawling.clone(),
        }
    }

    pub fn prone(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_prone.clone(),
            Side::B => self.side_b_prone.clone(),
        }
    }

    pub fn dead_prone(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_dead_prone.clone(),
            Side::B => self.side_b_dead_prone.clone(),
        }
    }

    #[allow(unused)]
    pub fn hurt_prone(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_hurt_prone.clone(),
            Side::B => self.side_b_hurt_prone.clone(),
        }
    }
}

#[derive(Debug, Constructor)]
pub struct SoldierAnimationInfos {
    side: Side,
    status: oc_individual::Status,
    gesture: oc_individual::Gesture,
}

impl IntoAnimation<SoldierAnimations> for SoldierAnimationInfos {
    fn animation(&self, animations: &SoldierAnimations) -> Handle<Animation> {
        let SoldierAnimationInfos {
            side,
            gesture,
            status,
        } = self;

        match status {
            oc_individual::Status::Operational => match gesture.body {
                oc_individual::BodyGesture::StandUp(_) => animations.stand_up(*side),
                oc_individual::BodyGesture::Walking(_) => animations.walking(*side),
                oc_individual::BodyGesture::Running(_) => animations.running(*side),
                oc_individual::BodyGesture::Crawling(_) => animations.crawling(*side),
                oc_individual::BodyGesture::Prone(_) => animations.prone(*side),
            },
            oc_individual::Status::Dead => animations.dead_prone(*side),
        }
    }
}
