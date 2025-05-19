use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex},
    time::Instant,
};
use rayon::prelude::*;
use crate::graph::{
    betweenness::Betweenness, edge::Edge, path::{Path, Paths}, undirected::{Community, UndirectedGraph}, utils::Utils, vertices::{VertexData, VerticesData}
};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    pub fn betweenness(&self) {
        let mut graph = self.clone();
        let mut generated_communities: HashMap<i32, Vec<Community<T>>> = HashMap::new();
        let vertices: Vec<T> = self.vertices.iter().cloned().collect();

        let mut counter = 0;
        while graph.has_edges() {
            let start_iter = Instant::now();
            let betweenness = Arc::new(Mutex::new(Betweenness::default()));

            // Calcula o betweenness em paralelo para cada vértice
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
                from: edge_with_biggest_betweenness.from.clone(),
                to: edge_with_biggest_betweenness.to.clone(),
            });
            graph.remove_edge(&Edge {
                from: edge_with_biggest_betweenness.to.clone(),
                to: edge_with_biggest_betweenness.from.clone(),
            });

            // Registra a divisão da comunidade
            let communities = graph.get_communities();
            if !generated_communities.contains_key(&(communities.len() as i32)) {
                Utils::persist_communities(
                    communities.clone(),
                    format!(
                        "{}_{}",
                        communities.len(),
                        self.get_modularity(graph.get_communities())
                    ),
                );
            }
            generated_communities
                .entry(communities.len() as i32)
                .or_insert(communities.clone());

            println!("General Time {}: {:?}", counter, start_iter.elapsed());
            println!();
            counter += 1;
        }
    }
}
