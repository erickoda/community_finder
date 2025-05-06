pub mod file;
pub mod graph;

use file::File;
use graph::Graph;

fn main() {
    let graph = Graph::<String>::from(File::read("got-edges.txt"));
    println!("{:?}", graph);
    graph.betweenness();
}
