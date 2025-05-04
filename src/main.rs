pub mod file;
pub mod graph;

use file::File;
use graph::Graph;

fn main() {
    let graph = Graph::<i32>::from(File::read("scale_free_network.txt"));
    println!("{:#?}", graph.vertices);
    println!("{:#?}", graph.adjacency);
    let betweenness = graph.betweenness();
}
