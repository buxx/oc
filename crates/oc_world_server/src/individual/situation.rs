pub struct Situation {
    pub enemy_visible: bool,
}

impl Situation {
    pub fn imply_hide(&self) -> bool {
        self.enemy_visible
    }
}
