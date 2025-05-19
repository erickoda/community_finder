use crate::graph::{
    edge::Edge,
    undirected::{Community, UndirectedGraph},
    utils::Utils,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    time::Instant,
};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    pub fn betweenness(&self) -> HashMap<usize, Vec<HashSet<T>>> {
        let mut graph = self.clone();
        let mut generated_communities: HashMap<usize, Vec<Community<T>>> = HashMap::new();

        let mut counter = 0;
        while graph.has_edges() {
            let start_iter = Instant::now();
            let edges_betweenness = self.get_edges_betweenness();

            if edges_betweenness.get_max().is_none() {
                break;
            }

            let edge_with_biggest_betweenness = edges_betweenness.get_max().unwrap().0.clone();

            // Remover a Edge
            graph.remove_edge(&Edge {
                from: edge_with_biggest_betweenness.from.clone(),
                to: edge_with_biggest_betweenness.to.clone(),
            });

            // Registra a divisão da comunidade
            let communities = graph.get_communities();
            if !generated_communities.contains_key(&(communities.len())) {
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
                .entry(communities.len())
                .or_insert(communities.clone());

            println!("General Time {}: {:?}", counter, start_iter.elapsed());
            println!();
            counter += 1;
        }

        generated_communities
    }
}
