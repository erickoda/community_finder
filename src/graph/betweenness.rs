use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Default)]
pub struct Betweenness<T> {
    pub edges: HashMap<(T, T), f64>,
}

impl<T> Betweenness<T>
where
    T: Eq + Hash + Clone,
{
    pub fn get_max(&self) -> (T, T) {
        self.edges
            .iter()
            .max_by(|x, y| x.1.total_cmp(y.1))
            .unwrap()
            .0
            .clone()
    }

    pub fn sum(&mut self, x: &Self) {
        for (edge, temp_value) in x.edges.iter() {
            if let Some(current_value) = self.edges.get_mut(&edge) {
                *current_value += temp_value;
                continue;
            }

            if let Some(current_value) = self.edges.get_mut(&(edge.1.clone(), edge.0.clone())) {
                *current_value += temp_value;
                continue;
            }

            self.insert_edge(edge.clone(), *temp_value);
        }
    }

    pub fn contains(&self, key: &(T, T)) -> bool {
        self.edges.get(key).is_some()
    }

    pub fn insert_edge(&mut self, key: (T, T), value: f64) {
        self.edges.insert(key, value);
    }

    pub fn sum_of_bellow_edges(&self, to_vertex: T) -> f64 {
        // Mudar a estrutura do Edges betweenness => o código
        // abaixo não é performático
        self.edges
            .iter()
            .filter(|((_, to), _)| *to == to_vertex)
            .fold(0., |acc, crr| acc + *crr.1)
    }
}
