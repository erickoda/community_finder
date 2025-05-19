pub mod betweenness;
pub mod hierarchical_growth;
pub mod newmans_modularity_clustering;

use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};

use super::{Community, UndirectedGraph};

impl<T> UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
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
}
