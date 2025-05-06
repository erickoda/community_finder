use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

#[derive(Default)]
pub struct Paths<T>(VecDeque<Path<T>>);

impl<T> Paths<T>
where
    T: Eq + Hash + Clone,
{
    pub fn push_back(&mut self, path: Path<T>) {
        self.0.push_back(path);
    }

    pub fn pop_back(&mut self) -> Option<Path<T>> {
        self.0.pop_back()
    }

    pub fn push_front(&mut self, path: Path<T>) {
        self.0.push_front(path);
    }
}

#[derive(Default, Debug, Clone)]
pub struct Path<T> {
    ordered: Vec<T>,
    vertices: HashSet<T>,
}

impl<T> Path<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(vertex: T) -> Self {
        let ordered = vec![vertex.clone()];
        Self {
            ordered: ordered.clone(),
            vertices: ordered.iter().cloned().collect(),
        }
    }

    pub fn push(&mut self, vertex: T) {
        self.ordered.push(vertex.clone());
        self.vertices.insert(vertex);
    }

    pub fn get_last_vertex(&self) -> &T {
        &self.ordered[self.ordered.len() - 1]
    }

    pub fn contains(&self, vertex: &T) -> bool {
        self.vertices.contains(vertex)
    }

    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn revert_path(&mut self) {
        self.ordered.reverse();
    }

    pub fn get(&self, i: usize) -> T {
        self.ordered[i].clone()
    }
}

impl<T> From<Vec<T>> for Path<T>
where
    T: Eq + Hash + Clone,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            ordered: value.clone(),
            vertices: value.iter().cloned().collect(),
        }
    }
}
