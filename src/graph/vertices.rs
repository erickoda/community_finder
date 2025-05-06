use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug)]
pub struct VerticesData<T>(HashMap<T, VertexData>);

impl<T> VerticesData<T>
where
    T: Eq + Hash + Clone,
{
    pub fn insert(&mut self, key: T, value: VertexData) {
        self.0.insert(key, value);
    }

    pub fn get_mut(&mut self, vertex: &T) -> Option<&mut VertexData> {
        self.0.get_mut(vertex)
    }
}
#[derive(Debug)]
pub struct VertexData {
    pub score: i32,
    pub distance: i32,
}

impl VertexData {
    pub fn new(score: i32, distance: i32) -> Self {
        Self { score, distance }
    }
}
