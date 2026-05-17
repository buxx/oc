use derive_more::Deref;

#[derive(Debug, Clone, Copy, Deref, PartialEq)]
pub struct Opacity(pub f32);

#[derive(Debug, Clone, Copy, Deref, PartialEq)]
pub struct CumulatedOpacity(pub f32);
