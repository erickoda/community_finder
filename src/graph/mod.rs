mod betweenness;
mod path;
mod utils;
mod vertices;

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
};

use betweenness::Betweenness;
use path::{Path, Paths};
use utils::Utils;
use vertices::{VertexData, VerticesData};

#[derive(Default, Debug, Clone)]
pub struct Graph<T> {
    pub vertices: HashSet<T>,
    pub adjacency: HashMap<T, Vec<T>>,
}

type Community<T> = HashSet<T>;

impl<T> Graph<T>
where
    T: Eq + Hash + Clone + Debug + Display + Default,
{
    pub fn new() -> Self {
        Graph {
            vertices: HashSet::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn push_vertex(&mut self, vertex: T) {
        self.vertices.insert(vertex);
    }

    pub fn push_edge(&mut self, from: T, to: T) {
        let adjacent_to_from = self.adjacency.entry(from).or_default();
        adjacent_to_from.push(to);
    }

    pub fn remove_edge(&mut self, from: T, to: T) {
        if let Some(adjacency) = self.adjacency.get_mut(&from) {
            if let Some(position) = adjacency.iter().position(|vertex| *vertex == to) {
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

    // Retorna as comunidades
    pub fn get_communities(&self) -> Vec<Community<T>> {
        let mut visited = HashSet::new();
        let mut communities = Vec::new();

        for vertex in &self.vertices {
            if visited.contains(vertex) {
                continue;
            }

            let mut stack = vec![vertex.clone()];
            let mut community = HashSet::new();

            while let Some(current) = stack.pop() {
                if !visited.insert(current.clone()) {
                    continue;
                }

                community.insert(current.clone());

                if let Some(neighbors) = self.adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }

                for (other, neighbors) in &self.adjacency {
                    if neighbors.contains(&current) && !visited.contains(other) {
                        stack.push(other.clone());
                    }
                }
            }

            communities.push(community);
        }

        communities
    }

    pub fn betweenness(&self) {
        let mut graph = self.clone();
        let mut generated_communities: HashMap<i32, Vec<Community<T>>> = HashMap::new();
        let vertices: Vec<T> = self.vertices.iter().cloned().collect();

        while graph.has_edges() {
            let mut current_paths_queue: Paths<T> = Paths::default();
            let mut dead_end_paths: Paths<T> = Paths::default(); // Registra os caminhos que não possuem saída
            let mut betweenness: Betweenness<T> = Betweenness::default();

            for vertex in vertices.clone() {
                current_paths_queue.push(Path::new(vertex.clone()));

                let mut vertices_data: VerticesData<T> = VerticesData::default();
                vertices_data.insert(vertex.clone(), VertexData::new(1, 0));

                /*
                 *  Implementação de uma BFS para encontrar os menos caminhos e calcular os scores
                 */
                while let Some(last_path) = current_paths_queue.pop() {
                    let last_vertex = last_path.get_last_vertex().clone();
                    let neighbourhood = graph.adjacency.get(&last_vertex);

                    // Verificar se a vizinhança é vazia
                    if neighbourhood.is_none() {
                        dead_end_paths.push(last_path.clone());
                        continue;
                    }

                    let mut neighbourhood = neighbourhood.unwrap().clone();

                    // Filtra os vizinhos que já estão no caminho
                    neighbourhood.retain(|neighbour| !last_path.contains(neighbour));

                    // Filtra os vizinhos que estão numa distância maior que a mínima
                    neighbourhood.retain(|neighbour| {
                        if let Some(vertex) = vertices_data.get_mut(&neighbour) {
                            // Verificar se o caminho também é ótimo
                            if vertex.distance == last_path.len() as i32 {
                                return true;
                            } else {
                                return false;
                            }
                        } else {
                            return true;
                        }
                    });

                    if neighbourhood.is_empty() {
                        dead_end_paths.push(last_path.clone());
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
                        new_path.push(neighbour.clone());
                        current_paths_queue.insert(0, new_path);
                    }
                }

                /*
                 *  A partir das folhas, deve-se calcular o betweenness com base nos scores
                 *  gerados.
                 */
                let mut temp_betweenness: Betweenness<T> = Betweenness::default();
                while let Some(mut biggest_path) = dead_end_paths.get_biggest_path() {
                    if biggest_path.len() == 1 {
                        continue;
                    }

                    biggest_path.revert_path();

                    if !temp_betweenness
                        .contains(&(biggest_path.get(0).clone(), biggest_path.get(1).clone()))
                    {
                        let score_i = vertices_data.get_score(&biggest_path.get(1)) as f64;
                        let score_j = vertices_data.get_score(&biggest_path.get(0)) as f64;

                        let bellow_neighbourhood_score_sum =
                            temp_betweenness.sum_of_bellow_edges(biggest_path.get(0));

                        temp_betweenness.insert_edge(
                            (biggest_path.get(0).clone(), biggest_path.get(1).clone()),
                            (1. + bellow_neighbourhood_score_sum) * (score_i / score_j),
                        );
                    }

                    biggest_path.remove(0);
                    biggest_path.revert_path();

                    dead_end_paths.push(Path::from(biggest_path));
                }

                betweenness.sum(&temp_betweenness);
            }

            // Calcular o maior betweenness
            let edge_with_biggest_betweenness = betweenness.get_max();

            // Remover a Edge
            graph.remove_edge(
                edge_with_biggest_betweenness.0.clone(),
                edge_with_biggest_betweenness.1.clone(),
            );
            graph.remove_edge(
                edge_with_biggest_betweenness.1.clone(),
                edge_with_biggest_betweenness.0.clone(),
            );

            // Registra a divisão da comunidade
            let communities = graph.get_communities();
            generated_communities
                .entry(communities.len() as i32)
                .or_insert(communities);
        }

        for communities in generated_communities {
            Utils::persist_communities(communities.1, communities.0.to_string());
        }
    }
}

impl<T> From<Vec<[i32; 2]>> for Graph<T>
where
    T: Eq + Hash + From<i32> + Clone + Debug + Display + Default,
{
    fn from(pairs: Vec<[i32; 2]>) -> Self {
        let mut graph = Graph::new();

        for [from, to] in pairs {
            graph.push_edge(T::from(from), T::from(to));
            graph.push_vertex(T::from(to));
            graph.push_vertex(T::from(from));
        }

        graph
    }
}
