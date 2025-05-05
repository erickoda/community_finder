use std::{collections::HashSet, hash::Hash};

#[derive(Default)]
pub struct Paths<Vertex>(Vec<Path<Vertex>>);

impl<Vertex> Paths<Vertex>
where
    Vertex: Eq + Hash + Clone,
{
    pub fn push(&mut self, path: Path<Vertex>) {
        self.0.push(path);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop(&mut self) -> Option<Path<Vertex>> {
        self.0.pop()
    }

    pub fn insert(&mut self, index: usize, path: Path<Vertex>) {
        self.0.insert(index, path);
    }

    pub fn get_biggest_path(&mut self) -> Option<Path<Vertex>> {
        if let Some((i, _)) = self.0.iter().enumerate().max_by_key(|(_, path)| path.len()) {
            return Some(self.0.swap_remove(i));
        }

        None
    }
}

#[derive(Default, Debug, Clone)]
pub struct Path<Vertex> {
    ordered: Vec<Vertex>,
    vertices: HashSet<Vertex>,
}

impl<Vertex> Path<Vertex>
where
    Vertex: Eq + Hash + Clone,
{
    pub fn new(vertex: Vertex) -> Self {
        let ordered = vec![vertex.clone()];
        Self {
            ordered: ordered.clone(),
            vertices: ordered.iter().cloned().collect(),
        }
    }

    pub fn push(&mut self, vertex: Vertex) {
        self.ordered.push(vertex.clone());
        self.vertices.insert(vertex);
    }

    pub fn remove(&mut self, position: usize) {
        self.ordered.remove(position);
    }

    pub fn get_last_vertex(&self) -> &Vertex {
        &self.ordered[self.ordered.len() - 1]
    }

    pub fn contains(&self, vertex: &Vertex) -> bool {
        self.vertices.contains(vertex)
    }

    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn revert_path(&mut self) {
        self.ordered.reverse();
    }

    pub fn get(&self, i: usize) -> Vertex {
        self.ordered[i].clone()
    }
}

impl<Vertex> From<Vec<Vertex>> for Path<Vertex>
where
    Vertex: Eq + Hash + Clone,
{
    fn from(value: Vec<Vertex>) -> Self {
        Self {
            ordered: value.clone(),
            vertices: value.iter().cloned().collect(),
        }
    }
}
