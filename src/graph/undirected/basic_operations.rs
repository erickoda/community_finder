use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::graph::edge::Edge;

use super::{Community, UndirectedGraph};

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
        if let Some(adjacency) = self.adjacency.get_mut(&edge.from) {
            if let Some(position) = adjacency.iter().position(|vertex| *vertex == edge.to) {
                adjacency.swap_remove(position);
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
                let neighbourhood_of_vertex = self.get_neighbourhood(vertex);
                if let Some(neighbourhood) = neighbourhood_of_vertex {
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

    pub fn get_number_of_triangles(&self, neighbourhood: &Vec<T>) -> usize {
        let mut number_of_triangles = 0;

        for (i, neighbour_u) in neighbourhood.iter().enumerate() {
            for neighbour_v in neighbourhood.iter().skip(i) {
                let u_neighbourhood = self.get_neighbourhood(neighbour_u);

                if let Some(u_neighbourhood) = u_neighbourhood {
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
}
