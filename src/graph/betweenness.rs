use std::{collections::HashMap, hash::Hash};

use super::edge::Edge;

#[derive(Debug, Clone, Default)]
pub struct Betweenness<T: Eq + Hash> {
    pub edges: HashMap<Edge<T>, f64>,
}

impl<T> Betweenness<T>
where
    T: Eq + Hash + Clone,
{
    pub fn get_max(&self) -> &Edge<T> {
        self.edges
            .iter()
            .max_by(|x, y| x.1.total_cmp(y.1))
            .unwrap()
            .0
    }

    pub fn sum(&mut self, x: &Self) {
        for (edge, temp_value) in x.edges.iter() {
            if let Some(current_value) = self.edges.get_mut(edge) {
                *current_value += temp_value;
                continue;
            }

            if let Some(current_value) = self.edges.get_mut(&Edge {
                from: edge.clone().to,
                to: edge.clone().from,
            }) {
                *current_value += temp_value;
                continue;
            }

            self.insert_edge(edge.clone(), *temp_value);
        }
    }

    pub fn contains(&self, key: &Edge<T>) -> bool {
        self.edges.contains_key(key)
    }

    pub fn insert_edge(&mut self, key: Edge<T>, value: f64) {
        self.edges.insert(key, value);
    }

    pub fn sum_of_bellow_edges(&self, to_vertex: T) -> f64 {
        // Mudar a estrutura do Edges betweenness => o código
        // abaixo não é performático
        self.edges
            .iter()
            .filter(|(edge, _)| edge.to == to_vertex)
            .fold(0., |acc, crr| acc + *crr.1)
    }
}
