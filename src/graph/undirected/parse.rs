use super::UndirectedGraph;
use crate::graph::edge::Edge;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

impl<T> From<Vec<[T; 2]>> for UndirectedGraph<T>
where
    T: Send + Sync + Eq + Hash + Clone + Debug + Display + Default,
{
    fn from(pairs: Vec<[T; 2]>) -> Self {
        let mut graph = UndirectedGraph::new();

        for [from, to] in pairs {
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
