pub trait WithIds<I, T> {
    fn with_ids(&self) -> Vec<(I, T)>;
}
