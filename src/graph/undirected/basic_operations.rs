use super::{Community, UndirectedGraph};
use crate::graph::{
    betweenness::Betweenness,
    edge::Edge,
    path::{Path, Paths},
    vertices::{VertexData, VerticesData},
};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex},
};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    pub fn new() -> Self {
        Self {
            vertices: HashSet::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn push_vertex(&mut self, vertex: T) {
        self.vertices.insert(vertex);
    }

    pub fn push_edge(&mut self, edge: &Edge<T>) {
        self.adjacency
            .entry(edge.from.clone())
            .and_modify(|adjacency| {
                if !adjacency.contains(&edge.to.clone()) {
                    adjacency.push(edge.to.clone())
                }
            })
            .or_insert(vec![edge.to.clone()]);
    }

    pub fn remove_edge(&mut self, edge: &Edge<T>) {
        if let Some(neighbourhood) = self.adjacency.get_mut(&edge.from) {
            if let Some(position) = neighbourhood.iter().position(|vertex| *vertex == edge.to) {
                neighbourhood.swap_remove(position);
            }
        }

        if let Some(neighbourhood) = self.adjacency.get_mut(&edge.to) {
            if let Some(position) = neighbourhood.iter().position(|vertex| *vertex == edge.from) {
                neighbourhood.swap_remove(position);
            }
        }
    }

    pub fn has_edges(&self) -> bool {
        for adjacency in self.adjacency.clone() {
            if !adjacency.1.is_empty() {
                return true;
            }
        }

        false
    }

    pub fn get_neighbourhood(&self, vertex: &T) -> Option<&Vec<T>> {
        self.adjacency.get(vertex)
    }

    pub fn get_neighbourhood_from_community(&self, community: &HashSet<T>) -> HashSet<T> {
        let mut neighbourhood_community: HashSet<T> = HashSet::new();

        for vertex in community {
            if let Some(vertex_neighbourhood) = self.get_neighbourhood(vertex) {
                let vertex_neighbourhood_set: HashSet<T> =
                    vertex_neighbourhood.iter().cloned().collect();

                neighbourhood_community = neighbourhood_community
                    .union(&vertex_neighbourhood_set)
                    .cloned()
                    .collect();
            }
        }

        neighbourhood_community
    }

    pub fn get_total_of_edges(&self) -> usize {
        self.adjacency.iter().fold(0, |acc, crr| acc + crr.1.len())
    }

    pub fn get_modularity(&self, communities: Vec<Community<T>>) -> f64 {
        let mut modularity_value = 0.;
        let total_of_edges = self.get_total_of_edges() as f64;

        for community in communities.iter() {
            let mut number_of_intra_community_links = 0;
            let mut sum_of_degrees = 0;
            for vertex in community.iter() {
                if let Some(neighbourhood_of_vertex) = self.get_neighbourhood(vertex) {
                    let neighbourhood = neighbourhood_of_vertex;
                    number_of_intra_community_links += neighbourhood
                        .iter()
                        .filter(|n| community.contains(n))
                        .count();
                    sum_of_degrees += neighbourhood.len();
                }
            }

            modularity_value += (number_of_intra_community_links as f64 / total_of_edges)
                - (sum_of_degrees as f64 / total_of_edges).powf(2.);
        }

        modularity_value
    }

    pub fn get_number_of_triangles(&self, neighbourhood: &[T]) -> usize {
        let mut number_of_triangles = 0;

        for (i, neighbour_u) in neighbourhood.iter().enumerate() {
            for neighbour_v in neighbourhood.iter().skip(i) {
                if let Some(u_neighbourhood) = self.get_neighbourhood(neighbour_u) {
                    let u_neighbourhood = u_neighbourhood;
                    let u_neighbourhood_set: HashSet<&T> = u_neighbourhood.iter().collect();
                    if neighbour_u != neighbour_v && u_neighbourhood_set.contains(neighbour_v) {
                        number_of_triangles += 1;
                        continue;
                    }
                }
            }
        }

        number_of_triangles
    }

    pub fn get_clustering_coefficients(&self) -> Vec<(&T, f64)> {
        let mut coefficients: Vec<(&T, f64)> = Vec::new();

        for vertex in self.vertices.iter() {
            if let Some(neighbourhood) = self.get_neighbourhood(vertex) {
                if neighbourhood.len() <= 1 {
                    coefficients.push((vertex, 0.));
                    continue;
                }

                let number_of_connected_triples: usize =
                    (neighbourhood.len() * (neighbourhood.len() - 1)) / 2;
                let number_of_triangles: usize = self.get_number_of_triangles(neighbourhood);

                coefficients.push((
                    vertex,
                    number_of_triangles as f64 / number_of_connected_triples as f64,
                ));
            }
        }

        coefficients
    }

    pub fn get_highest_clustering_coefficients(&self) -> (&T, f64) {
        match self
            .get_clustering_coefficients()
            .iter()
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            Some(coefficient) => *coefficient,
            None => panic!("ERROR: NO CLUSTERING COEFFICIENT WAS FOUND! YOUR GRAPH IS EMPTY"),
        }
    }

    pub fn get_edges(&self) -> Vec<Edge<T>> {
        let mut edges: Vec<Edge<T>> = vec![];
        for element in self.adjacency.clone() {
            for neighbour in element.1 {
                edges.push(Edge {
                    from: element.0.clone(),
                    to: neighbour,
                });
            }
        }
        edges
    }

    pub fn get_shortest_paths_starting_with(&self, vertex: &T) -> Paths<T> {
        let mut open_paths_queue: Paths<T> = Paths::default();
        let mut dead_end_paths: Paths<T> = Paths::default();
        let mut vertices_data: VerticesData<T> = VerticesData::default();

        open_paths_queue.push_back(Path::new(vertex.clone()));
        vertices_data.insert(vertex.clone(), VertexData::new(1, 0));

        /*
         *  Implementação de uma BFS para encontrar os menos caminhos e calcular os scores
         */
        while let Some(last_path) = open_paths_queue.pop_back() {
            let last_vertex = last_path.get_last_vertex();
            let mut neighbourhood = self.adjacency.get(last_vertex).cloned().unwrap_or_default();

            // Filtra os vizinhos que já estão no caminho
            neighbourhood.retain(|neighbour| !last_path.contains(neighbour));

            // Filtra os vizinhos que estão numa distância maior que a mínima
            neighbourhood.retain(|neighbour| {
                if let Some(vertex) = vertices_data.get_mut(neighbour) {
                    // Verificar se o caminho também é ótimo
                    vertex.distance == last_path.len() as i32
                } else {
                    true
                }
            });

            if neighbourhood.is_empty() {
                dead_end_paths.push_back(last_path);
                continue;
            }

            // Adiciona os vizinhos válidos no caminho e recalcula o score se necessário
            for neighbour in neighbourhood {
                match vertices_data.get_mut(&neighbour) {
                    // Verificar se já chegou nesse nó por outro caminho
                    Some(data) => {
                        data.score += 1;
                    }

                    // Caso o vértice ainda não tenha sido atingido
                    None => {
                        vertices_data.insert(
                            neighbour.clone(),
                            VertexData::new(1, last_path.len() as i32),
                        );
                    }
                }

                let mut new_path = last_path.clone();
                new_path.push(neighbour);
                open_paths_queue.push_front(new_path);
            }
        }

        dead_end_paths
    }

    pub fn get_edges_betweenness(&self) -> Betweenness<T> {
        let betweenness = Arc::new(Mutex::new(Betweenness::default()));

        self.vertices.par_iter().for_each(|vertex| {
            let mut dead_end_paths: Paths<T> = self.get_shortest_paths_starting_with(vertex); // Registra os caminhos que não possuem saída

            /*
             *  A partir das folhas, deve-se calcular o betweenness com base nos scores
             *  gerados.
             */
            let mut temp_betweenness: Betweenness<T> = Betweenness::default();
            while let Some(mut path) = dead_end_paths.pop_back() {
                path.revert_path();

                for i in 0..path.len() - 1 {
                    let edge = Edge {
                        from: path.get(i),
                        to: path.get(i + 1),
                    };

                    temp_betweenness
                        .values
                        .entry(edge)
                        .and_modify(|value| *value += i as f64 + 1.)
                        .or_insert(1.);
                }
            }
            let mut global = betweenness.lock().unwrap();
            global.sum(&temp_betweenness);
        });

        betweenness.lock().unwrap().clone()
    }
}
