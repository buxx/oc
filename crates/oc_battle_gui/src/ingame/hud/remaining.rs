use std::time::Duration;

use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct UpdateRemaining(pub Duration);

#[derive(Component, Default, Clone)]
pub struct RemainingText;

pub fn remaining() -> impl Scene {
    bsn! {
        RemainingText
        Text("--:--")
        TextFont { font_size: px(12) }
        Pickable::IGNORE
    }
}

pub fn on_update_remaining(
    event: On<UpdateRemaining>,
    mut text: Single<&mut Text, With<RemainingText>>,
) {
    let secs = event.0.as_secs();
    text.0 = format!("{:02}:{:02}", secs / 60, secs % 60);
}
