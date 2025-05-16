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
        //let communities = partitioned_graph.get_communities();

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

    pub fn hierarchical_growth(&self) {
        let vertices: Vec<&T> = self.vertices.iter().collect();
        let mut graph = self.clone();

        for (i, _) in vertices.iter().enumerate() {
            let time_main_loop = Instant::now();
            let vertex_with_highest_clustering_coefficient =
                graph.get_highest_clustering_coefficients().0;
            let mut community = vec![vertex_with_highest_clustering_coefficient.clone()];
            let mut has_grown = true;

            while has_grown {
                has_grown = false;

                let zero_block = Instant::now();
                let first_neighbourhood = graph
                    .get_neighbourhood_from_community(&community.clone().into_iter().collect());
                // println!("zero_block: {:?}", zero_block.elapsed());

                // Talvez dê para paralelizar esta parte
                for neighbour in first_neighbourhood.clone() {
                    let first_block = Instant::now();
                    let community_set: HashSet<T> = community.iter().cloned().collect();
                    let union_set: HashSet<T> =
                        community_set.union(&first_neighbourhood).cloned().collect();
                    let neighbourhood_of_neighbour: &Vec<T> =
                        graph.get_neighbourhood(&neighbour).unwrap();
                    // println!("first_block: {:?}", first_block.elapsed());

                    let second_block = Instant::now();
                    /*
                     *  Number of links of the vertex i with vertices belonging to community and
                     *  with vertices in the first neighborhood
                     */
                    let kin1: usize = neighbourhood_of_neighbour
                        .iter()
                        .filter(|n| union_set.contains(*n))
                        .count();

                    /*
                     *  number of links between the vertex i and vertices in the remainder of the
                     *  network
                     */
                    let kout1 = neighbourhood_of_neighbour
                        .iter()
                        .filter(|n| !union_set.contains(*n))
                        .count();

                    /*
                     * Caso o vértice esteja mais conectado com a comunidade do que com o resto da
                     * rede, ele é adicionado para a comunidade
                     */
                    if kin1 >= kout1 {
                        community.push(neighbour.clone());
                        has_grown = true;
                        // println!("second_block: {:?}", second_block.elapsed());
                        continue;
                    }

                    // println!("second_block: {:?}", second_block.elapsed());

                    let third_block = Instant::now();
                    let second_neighborhood = graph.get_neighbourhood_from_community(
                        &union_set.clone().iter().cloned().collect(),
                    );

                    let mut kin2 = 0;
                    let mut kout2 = 0;

                    for node in &second_neighborhood {
                        if let Some(neighs) = graph.get_neighbourhood(node) {
                            for n in neighs {
                                if first_neighbourhood.contains(n) {
                                    kin2 += 1;
                                } else {
                                    kout2 += 1;
                                }
                            }
                        }
                    }

                    let alfa = 1.;

                    if kin2 as f64 > alfa * kout2 as f64 {
                        community.push(neighbour.clone());
                        has_grown = true;
                        println!("third_block: {:?}", third_block.elapsed());
                        continue;
                    }

                    // println!("third_block: {:?}", third_block.elapsed());
                    let fourth_block = Instant::now();

                    if let Some(neighbourhood_of_neighbour) =
                        graph.clone().get_neighbourhood(&neighbour)
                    {
                        for neighbour_of_neighbour in neighbourhood_of_neighbour {
                            if community.contains(neighbour_of_neighbour) {
                                // println!("from: {} to: {}", neighbour, neighbour_of_neighbour);
                                graph.remove_edge(&Edge {
                                    from: neighbour.clone(),
                                    to: neighbour_of_neighbour.clone(),
                                });
                                graph.remove_edge(&Edge {
                                    to: neighbour.clone(),
                                    from: neighbour_of_neighbour.clone(),
                                });
                            }
                        }
                    }

                    // println!("fourth_block: {:?}", fourth_block.elapsed());
                    // println!()
                }
            }

            println!("Time({}): {:?}", i, time_main_loop.elapsed());
        }

        let result = graph.get_communities();
        Utils::persist_communities(
            result.clone(),
            format!(
                "hierarchical/{}_{}",
                result.len(),
                dbg!(self.get_modularity(graph.get_communities()))
            ),
        );
    }

    pub fn get_quantity_of_communities(&self) -> usize {
        let mut visited: HashSet<T> = HashSet::new();
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

                if let Some(neighbours) = self.adjacency.get(&current) {
                    for neighbour in neighbours {
                        if !visited.contains(neighbour) {
                            stack.push(neighbour.clone());
                        }
                    }
                }
                for (other, neighbours) in &self.adjacency {
                    if neighbours.contains(&current) && !visited.contains(other) {
                        stack.push(other.clone());
                    }
                }
            }

            communities.push(community);
        }

        communities.len()
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

    pub fn newmans_modularity_clustering(&self) {
        let mut partitions: HashMap<usize, (Vec<Community<T>>, f64)> = HashMap::new();
        let graph_with_no_edges = Graph {
            vertices: self.vertices.clone(),
            adjacency: self
                .vertices
                .iter()
                .map(|vertice| (vertice.clone(), Vec::new()))
                .collect::<HashMap<T, Vec<T>>>(),
        };

        let mut communities = graph_with_no_edges.get_communities();
        let total_of_edges = self.get_total_of_edges() as f64;

        while communities.len() != 1 {
            let start = Instant::now();

            let mut highest_pair: f64 = f64::NEG_INFINITY;
            let mut best_pair: Option<(usize, usize)> = None;

            for (i, community_i) in communities.iter().take(communities.len() - 1).enumerate() {
                let community_i_neighbourhood = self.get_neighbourhood_from_community(community_i);
                for (j_offset, community_j) in communities.iter().skip(i + 1).enumerate() {
                    let j = j_offset + i + 1;
                    let delta_modularity = {
                        let mut e_ij = 0.;
                        let mut ai = 0.;
                        let mut aj = 0.;

                        for neighbour in &community_i_neighbourhood {
                            if community_j.contains(neighbour) {
                                e_ij += 1.;
                            }
                        }

                        for vertex in community_i {
                            let neighbourhood = self.get_neighbourhood(vertex);
                            if let Some(neighbourhood) = neighbourhood {
                                ai += neighbourhood.len() as f64;
                            }
                        }

                        for vertex in community_j {
                            let neighbourhood = self.get_neighbourhood(vertex);
                            if let Some(neighbourhood) = neighbourhood {
                                aj += neighbourhood.len() as f64;
                            }
                        }

                        2. * ((e_ij / total_of_edges) - ((ai * aj) / (total_of_edges.powf(2.))))
                    };

                    if delta_modularity > highest_pair {
                        highest_pair = delta_modularity;
                        best_pair = Some((i, j));
                    }
                }
            }

            if let Some((i, j)) = best_pair {
                let community_i = communities[i].clone();
                let community_j = communities[j].clone();
                communities.remove(j);
                communities.remove(i);
                let unified_communities: HashSet<T> =
                    community_i.union(&community_j).cloned().collect();
                communities.push(unified_communities);
                partitions.insert(
                    communities.len(),
                    (
                        communities.clone(),
                        self.get_modularity(communities.clone()),
                    ),
                );
            }

            println!("Time({}): {:?}", communities.len(), start.elapsed());
        }

        for communities in partitions.values() {
            Utils::persist_communities(
                communities.0.clone(),
                format!("{}_{}", communities.0.len(), communities.1),
            );
        }
    }

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
