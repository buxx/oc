use oc_root::opacity::Opacity;

pub struct Terrain {
    pub opacity: f32,
}

impl Terrain {
    // FIXME BS NOW: z tralala
    pub fn opacity(&self, _z: f32) -> Opacity {
        Opacity(self.opacity)
    }
}
