use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Default)]
pub struct Betweenness<T> {
    pub edges: HashMap<
        (
            T, /* Primeiro vértice do caminho atual */
            T, /* Vértice Atual */
            T, /* Vértice Posterior */
        ),
        f64,
    >,
}

impl<T> Betweenness<T>
where
    T: Eq + Hash + Clone,
{
    pub fn get_max(&self) -> (T, T) {
        self.calculate()
            .iter()
            .max_by(|x, y| x.1.total_cmp(y.1))
            .unwrap()
            .0
            .clone()
    }

    pub fn contains(&self, key: &(T, T, T)) -> bool {
        self.edges.get(key).is_some()
    }

    pub fn insert_edge(&mut self, key: (T, T, T), value: f64) {
        self.edges.insert(key, value);
    }

    pub fn sum_of_bellow_edges(&self, origin_vertex: T, to_vertex: T) -> f64 {
        // Mudar a estrutura do Edges betweenness => o código
        // abaixo não é performático
        self.edges
            .iter()
            .filter(|((origin, _, to), _)| *to == to_vertex && *origin == origin_vertex)
            .fold(0., |acc, crr| acc + *crr.1)
    }

    // Calcular o betweenness de cada edge
    fn calculate(&self) -> HashMap<(T, T), f64> {
        let mut betweenness_per_edge: HashMap<(T, T), f64> = HashMap::new();

        for ((_, from, to), betweenness_incomplete_value) in self.edges.clone() {
            if let Some(betweenness_value) =
                betweenness_per_edge.get_mut(&(from.clone(), to.clone()))
            {
                *betweenness_value += betweenness_incomplete_value;
                continue;
            }

            if let Some(betweenness_value) =
                betweenness_per_edge.get_mut(&(to.clone(), from.clone()))
            {
                *betweenness_value += betweenness_incomplete_value;
                continue;
            }

            betweenness_per_edge.insert((from.clone(), to.clone()), betweenness_incomplete_value);
        }

        betweenness_per_edge
    }
}
