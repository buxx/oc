use bevy::color::Color;

use crate::utils::IntoColor;

impl IntoColor for oc_individual::Status {
    fn color(&self) -> Color {
        match self {
            oc_individual::Status::Operational => Color::srgba(0., 1., 0., 1.0),
            oc_individual::Status::Dead => Color::srgba(1.0, 0., 0., 1.0),
        }
    }
}
