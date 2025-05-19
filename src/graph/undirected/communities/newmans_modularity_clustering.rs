use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    time::Instant,
};

use crate::{
    graph::{
        undirected::{Community, UndirectedGraph},
        utils::Utils,
    },
    utils::OrderedF64,
};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    pub fn newmans_modularity_clustering(&self) -> HashMap<usize, Vec<HashSet<T>>> {
        let total_of_edges = self.get_total_of_edges() as f64;
        let mut partitions: HashMap<usize, (Vec<Community<T>>, f64)> = HashMap::new();
        let mut communities: HashMap<usize, HashSet<T>> = HashMap::new();
        let mut delta_q: HashMap<(usize, usize), f64> = HashMap::new();
        let mut heap: BinaryHeap<(OrderedF64, usize, usize)> = BinaryHeap::new();
        let mut community_id = 0;
        let mut degrees: HashMap<usize, f64> = HashMap::new();
        let mut active_ids: HashSet<usize> = HashSet::new();
        let mut vertex_to_community: HashMap<&T, usize> = HashMap::new();

        for (vertex, neighbourhood) in &self.adjacency {
            communities.insert(community_id, HashSet::from([vertex.clone()]));
            degrees.insert(community_id, neighbourhood.len() as f64);
            active_ids.insert(community_id);
            vertex_to_community.insert(vertex, community_id);
            community_id += 1;
        }

        for (vertex, neighbourhood) in &self.adjacency {
            let community_i = vertex_to_community[vertex];
            for neighbour in neighbourhood {
                let community_j = vertex_to_community[neighbour];
                let key = if community_i < community_j {
                    (community_i, community_j)
                } else {
                    (community_j, community_i)
                };
                delta_q
                    .entry(key)
                    .and_modify(|value| *value += 1. / total_of_edges)
                    .or_insert(
                        (1. / total_of_edges)
                            - ((2.
                                * neighbourhood.len() as f64
                                * self.get_neighbourhood(neighbour).iter().len() as f64)
                                / total_of_edges.powf(2.)),
                    );
            }
        }

        for ((community_i, community_j), modularity) in &delta_q {
            heap.push((OrderedF64(*modularity), *community_i, *community_j));
        }

        while active_ids.len() > 1 {
            let highest = heap.pop();

            if let Some(highest) = highest {
                let start = Instant::now();

                if !active_ids.contains(&highest.1) || !active_ids.contains(&highest.2) {
                    continue;
                }

                let community_i = communities.get(&highest.1).unwrap();
                let community_j = communities.get(&highest.2).unwrap();
                let degree_community_i = degrees.get(&highest.1).unwrap();
                let degree_community_j = degrees.get(&highest.2).unwrap();
                let new_degree = *degree_community_i + *degree_community_j;
                let unified_communities: HashSet<T> =
                    community_i.union(community_j).cloned().collect();

                active_ids.remove(&highest.1);
                active_ids.remove(&highest.2);

                for active_id in &active_ids {
                    let e_ij = self
                        .get_neighbourhood_from_community(&unified_communities)
                        .intersection(communities.get(active_id).unwrap())
                        .collect::<HashSet<_>>()
                        .len() as f64
                        / total_of_edges;
                    let degree = *degrees.get(active_id).unwrap();
                    let new_delta_q =
                        2. * (e_ij - ((new_degree * degree) / (total_of_edges.powf(2.))));

                    heap.push((OrderedF64(new_delta_q), *active_id, community_id));
                }

                degrees.insert(community_id, new_degree);
                communities.insert(community_id, unified_communities);
                active_ids.insert(community_id);
                community_id += 1;

                let snapshot = active_ids
                    .iter()
                    .map(|id| communities[id].clone())
                    .collect::<Vec<_>>();
                let q = self.get_modularity(snapshot.clone());
                partitions.insert(self.vertices.len() - partitions.len(), (snapshot, q));
                println!("Time({}): {:?}", active_ids.len(), start.elapsed());
            }
        }

        for communities in partitions.values() {
            Utils::persist_communities(
                communities.0.clone(),
                format!("{}_{}", communities.0.len(), communities.1),
            );
        }

        partitions
            .iter()
            .map(|(key, value)| (*key, value.0.clone()))
            .collect::<HashMap<_, _>>()
    }
}
