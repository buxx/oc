use oc_root::WORLD_WIDTH;

#[derive(Debug, Clone, Copy)]
pub struct Xy(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub struct XyIndex(pub usize);

impl From<XyIndex> for Xy {
    fn from(XyIndex(i): XyIndex) -> Self {
        let x = i % WORLD_WIDTH;
        let y = i / WORLD_WIDTH;
        Self(x, y)
    }
}

impl From<Xy> for XyIndex {
    fn from(Xy(x, y): Xy) -> Self {
        Self(y * WORLD_WIDTH + x)
    }
}

impl From<Xy> for (usize, usize) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}
