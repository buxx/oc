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

// soldiers.png
// TODO: use generative macro to write less code (animations creations and functions to get sprite + animation)
#[derive(Debug, Resource)]
pub struct SoldierAnimations {
    sprite: Sprite,
    side_a_idle: Handle<Animation>,
    side_a_walking: Handle<Animation>,
    side_a_running: Handle<Animation>,
    side_a_crawling: Handle<Animation>,
    side_a_lying: Handle<Animation>,
    side_a_dead_lying: Handle<Animation>,
    #[allow(unused)]
    side_a_hurt_lying: Handle<Animation>,
    side_b_idle: Handle<Animation>,
    side_b_walking: Handle<Animation>,
    side_b_running: Handle<Animation>,
    side_b_crawling: Handle<Animation>,
    side_b_lying: Handle<Animation>,
    side_b_dead_lying: Handle<Animation>,
    #[allow(unused)]
    side_b_hurt_lying: Handle<Animation>,
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

        let side_a_idle = spritesheet
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
        let side_a_lying = spritesheet
            .create_animation()
            .add_row(3)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_a_dead_lying = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_a_hurt_lying = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();

        let side_a_idle = animations.add(side_a_idle);
        let side_a_walking = animations.add(side_a_walking);
        let side_a_running = animations.add(side_a_running);
        let side_a_crawling = animations.add(side_a_crawling);
        let side_a_lying = animations.add(side_a_lying);
        let side_a_dead_lying = animations.add(side_a_dead_lying);
        let side_a_hurt_lying = animations.add(side_a_hurt_lying);

        // FICME BS NOW: side b must be plus bas dans les rows
        let side_b_idle = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 0, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_walking = spritesheet
            .create_animation()
            .add_row(1)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_b_running = spritesheet
            .create_animation()
            .add_row(1)
            .set_duration(AnimationDuration::PerFrame(250))
            .build();
        let side_b_crawling = spritesheet
            .create_animation()
            .add_row(2)
            .set_duration(AnimationDuration::PerFrame(500))
            .build();
        let side_b_lying = spritesheet
            .create_animation()
            .add_row(3)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_dead_lying = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();
        let side_b_hurt_lying = spritesheet
            .create_animation()
            .add_horizontal_strip(0, 4, 2)
            .set_duration(AnimationDuration::PerFrame(1000))
            .build();

        let side_b_idle = animations.add(side_b_idle);
        let side_b_walking = animations.add(side_b_walking);
        let side_b_running = animations.add(side_b_running);
        let side_b_crawling = animations.add(side_b_crawling);
        let side_b_lying = animations.add(side_b_lying);
        let side_b_dead_lying = animations.add(side_b_dead_lying);
        let side_b_hurt_lying = animations.add(side_b_hurt_lying);

        let sprite = spritesheet
            .with_size_hint(IMAGE_WIDTH, IMAGE_HEIGHT)
            .sprite(atlas_layouts);

        Self {
            sprite,
            side_a_idle,
            side_a_walking,
            side_a_running,
            side_a_crawling,
            side_a_lying,
            side_a_dead_lying,
            side_a_hurt_lying,
            side_b_idle,
            side_b_walking,
            side_b_running,
            side_b_crawling,
            side_b_lying,
            side_b_dead_lying,
            side_b_hurt_lying,
        }
    }

    pub fn sprite(&self) -> Sprite {
        self.sprite.clone()
    }

    pub fn idle(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_idle.clone(),
            Side::B => self.side_b_idle.clone(),
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

    pub fn lying(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_lying.clone(),
            Side::B => self.side_b_lying.clone(),
        }
    }

    pub fn dead_lying(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_dead_lying.clone(),
            Side::B => self.side_b_dead_lying.clone(),
        }
    }

    #[allow(unused)]
    pub fn hurt_lying(&self, side: Side) -> Handle<Animation> {
        match side {
            Side::A => self.side_a_hurt_lying.clone(),
            Side::B => self.side_b_hurt_lying.clone(),
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
            oc_individual::Status::Operational => match gesture {
                oc_individual::Gesture::Idle(_) => animations.idle(*side),
                oc_individual::Gesture::Walking(_) => animations.walking(*side),
                oc_individual::Gesture::Running(_) => animations.running(*side),
                oc_individual::Gesture::Crawling(_) => animations.crawling(*side),
                oc_individual::Gesture::Lying(_) => animations.lying(*side),
            },
            oc_individual::Status::Dead => animations.dead_lying(*side),
        }
    }
}
