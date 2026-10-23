#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Morale {
    pub a: f32,
    pub b: f32,
}

impl Default for Morale {
    fn default() -> Self {
        Self { a: 100., b: 100. }
    }
}
