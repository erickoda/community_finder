mod betweenness;
mod edge;
mod path;
mod utils;
mod vertices;

use betweenness::Betweenness;
use edge::Edge;
use path::{Path, Paths};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
};
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
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
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

    pub fn remove_edge(&mut self, edge: &Edge<&T>) {
        if let Some(adjacency) = self.adjacency.get_mut(edge.from) {
            if let Some(position) = adjacency.iter().position(|vertex| vertex == edge.to) {
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

        let mut counter = 0;
        while graph.has_edges() {
            let start_iter = Instant::now();
            let betweenness = Arc::new(Mutex::new(Betweenness::default()));

            // It calculates the betweenness in parallel for each
            vertices.par_iter().for_each(|vertex| {
                let mut current_paths_queue: Paths<T> = Paths::default();
                let mut dead_end_paths: Paths<T> = Paths::default(); // Registra os caminhos que não possuem saída
                let mut vertices_data: VerticesData<T> = VerticesData::default();

                current_paths_queue.push_back(Path::new(vertex.clone()));
                vertices_data.insert(vertex.clone(), VertexData::new(1, 0));

                /*
                 *  Implementação de uma BFS para encontrar os menos caminhos e calcular os scores
                 */
                while let Some(last_path) = current_paths_queue.pop_back() {
                    let last_vertex = last_path.get_last_vertex();
                    let mut neighbourhood = graph
                        .adjacency
                        .get(last_vertex)
                        .cloned()
                        .unwrap_or_default();

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
                        current_paths_queue.push_front(new_path);
                    }
                }

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
                            .edges
                            .entry(edge)
                            .and_modify(|value| *value += i as f64 + 1.)
                            .or_insert(1.);
                    }
                }
                let mut global = betweenness.lock().unwrap();
                global.sum(&temp_betweenness);
            });

            if betweenness.lock().unwrap().get_max().is_none() {
                break;
            }

            let edge_with_biggest_betweenness =
                betweenness.lock().unwrap().get_max().unwrap().0.clone();

            // Remover a Edge
            graph.remove_edge(&Edge {
                from: &edge_with_biggest_betweenness.from,
                to: &edge_with_biggest_betweenness.to,
            });
            graph.remove_edge(&Edge {
                from: &edge_with_biggest_betweenness.to,
                to: &edge_with_biggest_betweenness.from,
            });

            // Registra a divisão da comunidade
            let communities = graph.get_communities();
            generated_communities
                .entry(communities.len() as i32)
                .or_insert(communities);

            println!("General Time {}: {:?}", counter, start_iter.elapsed());
            println!();
            counter += 1;
        }

        for communities in generated_communities {
            Utils::persist_communities(communities.1, communities.0.to_string());
        }
    }
}

impl<T> From<Vec<[T; 2]>> for Graph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    fn from(pairs: Vec<[T; 2]>) -> Self {
        let mut graph = Graph::new();

        for [from, to] in pairs {
            // Estou supondo que a rede é não direcionada
            graph.push_edge(&Edge {
                from: from.clone(),
                to: to.clone(),
            });
            graph.push_edge(&Edge {
                to: from.clone(),
                from: to.clone(),
            });
            graph.push_vertex(to);
            graph.push_vertex(from);
        }

        graph
    }
}
