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
    pub fn get_max(&self) -> Option<(&Edge<T>, &f64)> {
        self.edges.iter().max_by(|x, y| x.1.total_cmp(y.1))
        // .unwrap()
        // .0
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

    pub fn insert_edge(&mut self, key: Edge<T>, value: f64) {
        self.edges.insert(key, value);
    }
}
