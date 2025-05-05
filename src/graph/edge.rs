#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct Edge<T> {
    pub from: T,
    pub to: T,
}
