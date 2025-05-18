pub mod analyses;
pub mod file;
pub mod graph;
pub mod utils;

use std::time::Instant;

use file::File;
use graph::Graph;

fn main() {
    let graph = Graph::<String>::from(File::read("caveman_graph_2.txt"));
    println!("{:?}", graph);
    let hierarchical_time = Instant::now();
    //graph.hierarchical_growth();
    let hierarchical_elapsed = hierarchical_time.elapsed();
    let betweenness_time = Instant::now();
    // graph.betweenness();
    graph.newmans_modularity_clustering();
    println!("Betweenness Time: {:?}", betweenness_time.elapsed());
    println!("Hierarchical growth time: {:?}", hierarchical_elapsed);
}
