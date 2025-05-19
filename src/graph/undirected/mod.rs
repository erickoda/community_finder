pub mod basic_operations;
pub mod communities;
pub mod parse;

use std::collections::{HashMap, HashSet};

#[derive(Default, Debug, Clone)]
pub struct UndirectedGraph<T> {
    pub vertices: HashSet<T>,
    pub adjacency: HashMap<T, Vec<T>>,
}

pub type Community<T> = HashSet<T>;
