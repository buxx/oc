#[cfg(feature = "debug")]
use derive_more::Deref;

#[cfg(feature = "debug")]
#[derive(Debug, Deref)]
pub struct Scheduling<T>(pub Vec<T>);

#[cfg(feature = "debug")]
pub trait Schedule<C, T> {
    fn schedule(&self, ctx: C) -> Scheduling<T>;
}
