pub mod file;
pub mod graph;

use file::File;
use graph::Graph;

fn main() {
    let graph = Graph::<i32>::from(File::read("email-Eu-core.txt"));
    graph.betweenness();
}
