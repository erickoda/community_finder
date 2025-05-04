use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    fs::File,
    hash::Hash,
    io::Write,
};

#[derive(Default, Debug, Clone)]
pub struct Graph<Vertex> {
    pub vertices: HashSet<Vertex>,
    pub adjacency: HashMap<Vertex, Vec<Vertex>>,
}

struct Path<Vertex> {
    path: Vec<Vertex>,
    vertices: HashSet<Vertex>,
    last_removed_vertex: Vertex,
}

type Community<Vertex> = HashSet<Vertex>;

impl<Vertex> Graph<Vertex>
where
    Vertex: Eq + Hash + Clone + Debug + From<i32> + Display,
{
    pub fn new() -> Self {
        Graph {
            vertices: HashSet::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn push_vertex(&mut self, vertex: Vertex) {
        self.vertices.insert(vertex);
    }

    pub fn push_edge(&mut self, from: Vertex, to: Vertex) {
        let adjacent_to_from = self.adjacency.entry(from).or_default();
        adjacent_to_from.push(to);
    }

    pub fn remove_edge(&mut self, from: Vertex, to: Vertex) {
        if let Some(adjacency) = self.adjacency.get_mut(&from) {
            if let Some(position) = adjacency.iter().position(|vertex| *vertex == to) {
                adjacency.swap_remove(position);
                println!("Removed");
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
    pub fn get_communities(&self) -> Vec<Community<Vertex>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for vertex in &self.vertices {
            if visited.contains(vertex) {
                continue;
            }

            let mut stack = vec![vertex.clone()];
            let mut component = HashSet::new();

            while let Some(current) = stack.pop() {
                if !visited.insert(current.clone()) {
                    continue;
                }

                component.insert(current.clone());

                if let Some(neighbors) = self.adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }

            components.push(component);
        }

        components
    }

    pub fn betweenness(&self) {
        let mut graph = self.clone();
        let mut generated_communities: HashMap<i32, Vec<Community<Vertex>>> = HashMap::new();
        let vertices: Vec<Vertex> = self.vertices.iter().cloned().collect();

        loop {
            if !graph.has_edges() {
                break;
            }

            // println!("{:?}", graph);
            // thread::sleep(Duration::from_secs(1));

            let mut final_betweenness: HashMap<(Vertex, Vertex), f64> = HashMap::new();
            let mut queue: Vec<(Vec<Vertex>, HashSet<Vertex>)> = Vec::new();
            let mut edges_betweenness: HashMap<
                (
                    Vertex, /* Primeiro vertice do caminho atual */
                    Vertex, /*Vertice Atual*/
                    Vertex, /*Vertice Posterior*/
                ),
                f64,
            > = HashMap::new();
            let mut ended_paths: Vec<(Vec<Vertex>, HashSet<Vertex>)> = Vec::new(); // Registra os caminhos que não possuiem saída

            for vertex in vertices.clone() {
                let initial_neighbourhood = graph.adjacency.get(&vertex);

                // The First Value is the score and the second one is the distance from the current vertex
                let mut vertices_with_score_and_distance: HashMap<
                    Vertex,
                    (HashMap<Vertex, i32>, i32),
                > = HashMap::new();
                vertices_with_score_and_distance
                    .insert(vertex.clone(), (HashMap::from([(vertex.clone(), 1)]), 0));

                if let Some(initial_neighbourhood) = initial_neighbourhood {
                    for neighbour in initial_neighbourhood {
                        queue.push((
                            vec![vertex.clone(), neighbour.clone()],
                            HashSet::from([vertex.clone(), neighbour.clone()]),
                        ));
                        vertices_with_score_and_distance
                            .insert(neighbour.clone(), (HashMap::from([(vertex.clone(), 1)]), 1));
                    }
                }

                loop {
                    if queue.is_empty() {
                        /*
                         *  A partir das folhas, deve-se calcular o betweenness com base no wi e wj
                         *  gerados.
                         */
                        loop {
                            if ended_paths.is_empty() {
                                break;
                            }

                            let bigest_path = {
                                if let Some((i, _)) = ended_paths
                                    .iter()
                                    .enumerate()
                                    .max_by_key(|(_, path)| path.0.len())
                                {
                                    Some(ended_paths.swap_remove(i))
                                } else {
                                    None
                                }
                            }
                            .unwrap();

                            if bigest_path.0.len() == 1 {
                                continue;
                            }

                            //for i in 0..bigest_path.0.len() - 1 {
                            let mut reverted_path: Vec<Vertex> =
                                bigest_path.0.iter().rev().cloned().collect();

                            match edges_betweenness.get(&(
                                vertex.clone(),
                                reverted_path[0].clone(),
                                reverted_path[1].clone(),
                            )) {
                                Some(_) => {}
                                None => {
                                    let score_i = vertices_with_score_and_distance
                                        .get(&reverted_path[1])
                                        .unwrap()
                                        .0
                                        .iter()
                                        .fold(0, |acc, crr| acc + crr.1)
                                        as f64;
                                    let score_j = vertices_with_score_and_distance
                                        .get(&reverted_path[0])
                                        .unwrap()
                                        .0
                                        .iter()
                                        .fold(0, |acc, crr| acc + crr.1)
                                        as f64;
                                    let bellow_neighbourhood_score_sum = {
                                        // Mudar a estrutura do Edges betweenness => o código
                                        // abaixo não é peformático
                                        edges_betweenness
                                            .iter()
                                            .filter(|((origin, _, to), _)| {
                                                *to == reverted_path[0]
                                                    && *origin
                                                        == reverted_path[reverted_path.len() - 1]
                                            })
                                            .fold(0., |acc, crr| acc + *crr.1)
                                    };
                                    edges_betweenness.insert(
                                        (
                                            vertex.clone(),
                                            reverted_path[0].clone(),
                                            reverted_path[1].clone(),
                                        ),
                                        (1. + bellow_neighbourhood_score_sum) * (score_i / score_j),
                                    );
                                }
                            }

                            reverted_path.remove(0);
                            reverted_path.reverse();
                            ended_paths.push((
                                reverted_path.clone(),
                                reverted_path.iter().cloned().collect(),
                            ));
                        }

                        break;
                    }

                    let last_path = queue.pop().unwrap();
                    let last_vertex = last_path.0[last_path.0.len() - 1].clone();
                    let last_score = {
                        vertices_with_score_and_distance
                            .get(&last_vertex)
                            .unwrap()
                            .0
                            .clone()
                            .iter()
                            .fold(0, |acc, current| acc + *current.1)
                    };
                    let neighbourhood = graph.adjacency.get(&last_vertex);
                    // Verificar se:
                    // 1 - SE neighbour == 0, ENTÃO o NÓ é folha
                    // 2 - SE a vizinhança só contém nós que já estão no último caminho
                    // Resultado: se qualquer um dos dois for verdade, o caminho deve ser colocado na
                    // variável ended_paths
                    if neighbourhood.is_none() {
                        ended_paths.push(last_path.clone());
                        continue;
                    }

                    let mut neighbourhood = neighbourhood.unwrap().clone();
                    neighbourhood.retain(|neighbour| !last_path.1.contains(neighbour));

                    // Verificar se o ainda há caminhos para o nó atual
                    let is_leaf = {
                        let mut is_leaf = true;

                        for neighbour in neighbourhood.clone() {
                            if last_path.1.contains(&neighbour) {
                                continue;
                            }

                            if !vertices_with_score_and_distance.contains_key(&neighbour) {
                                is_leaf = false;
                                break;
                            }

                            if let Some((_, distance)) =
                                vertices_with_score_and_distance.get_mut(&neighbour)
                            {
                                if *distance == last_path.0.len() as i32 {
                                    is_leaf = false;
                                    break;
                                }
                            }
                        }

                        is_leaf
                    };

                    if is_leaf {
                        ended_paths.push(last_path);
                        continue;
                    }

                    for neighbour in neighbourhood {
                        if last_path.1.contains(&neighbour) {
                            continue;
                        }

                        match vertices_with_score_and_distance.get_mut(&neighbour) {
                            // Verificar se já chegou nesse nó
                            Some((score, distance)) => {
                                // Verificar se o caminho também é ótimo
                                if *distance == last_path.0.len() as i32 {
                                    score.insert(vertex.clone(), last_score);
                                } else {
                                    continue;
                                }
                            }

                            // Caso o vértice ainda não tenha sido atingido
                            None => {
                                vertices_with_score_and_distance.insert(
                                    neighbour.clone(),
                                    (
                                        HashMap::from([(last_vertex.clone(), 1)]),
                                        last_path.0.len() as i32,
                                    ),
                                );
                            }
                        }

                        let mut new_path = last_path.clone();
                        new_path.0.push(neighbour.clone());
                        new_path.1.insert(neighbour.clone());
                        queue.insert(0, new_path);
                    }
                }
            }

            // Calcular o betweenness de cada edge
            for ((_, from, to), betweenness) in edges_betweenness {
                if let Some(final_betweenness_value) =
                    final_betweenness.get_mut(&(from.clone(), to.clone()))
                {
                    *final_betweenness_value += betweenness;
                    continue;
                }

                if let Some(final_betweenness_value) =
                    final_betweenness.get_mut(&(to.clone(), from.clone()))
                {
                    *final_betweenness_value += betweenness;
                    continue;
                }

                final_betweenness.insert((from.clone(), to.clone()), betweenness);
            }

            // Calcular o maior betweenness
            let edge_with_biggest_betweenness = final_betweenness
                .iter()
                .max_by(|x, y| x.1.total_cmp(y.1))
                .unwrap()
                .0
                .clone();

            // Remover a Edge
            graph.remove_edge(
                edge_with_biggest_betweenness.0.clone(),
                edge_with_biggest_betweenness.1.clone(),
            );
            graph.remove_edge(
                edge_with_biggest_betweenness.1.clone(),
                edge_with_biggest_betweenness.0.clone(),
            );

            let communities = graph.get_communities();

            generated_communities
                .entry(communities.len() as i32)
                .or_insert(communities);
        }

        println!("GENERATED COMMUNITIES: {:#?}", generated_communities);

        for communities in generated_communities {
            Utils::persist_communities(communities.1, communities.0.to_string());
        }
    }
}

impl<Vertex> From<Vec<[i32; 2]>> for Graph<Vertex>
where
    Vertex: Eq + Hash + From<i32> + Clone + Debug + Display,
{
    fn from(pairs: Vec<[i32; 2]>) -> Self {
        let mut graph = Graph::new();

        for [from, to] in pairs {
            graph.push_edge(Vertex::from(from), Vertex::from(to));
            graph.push_vertex(Vertex::from(to));
            graph.push_vertex(Vertex::from(from));
        }

        graph
    }
}

struct Utils;

impl Utils {
    pub fn persist_communities<Vertex: Display>(
        communities: Vec<Community<Vertex>>,
        file_name: impl Into<String>,
    ) {
        let mut file = File::create(String::from("./out/") + &file_name.into() + ".txt")
            .expect("ERROR: FAILED TO PERSIST COMMUNITIES");

        for community in communities {
            for vertex in community {
                write!(file, "{} ", vertex)
                    .expect("ERROR: FAILED TO WRITE ON COMMUNITY PERSISTENCE");
            }

            writeln!(file).expect("ERROR: FAILED TO WRITE LINE ON COMMUNITY PERSISTENCE");
        }
    }
}
