use std::{collections::HashSet, fmt::{Debug, Display}, hash::Hash, time::Instant};

use crate::graph::{edge::Edge, undirected::UndirectedGraph, utils::Utils};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
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
}
