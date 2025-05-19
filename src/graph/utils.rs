use std::{fmt::Display, fs::File, io::Write};

use super::undirected::Community;

pub struct Utils;

impl Utils {
    pub fn persist_communities<T: Display>(
        communities: Vec<Community<T>>,
        file_name: impl Into<String>,
    ) {
        let mut file = File::create(String::from("./out/") + &file_name.into() + ".txt")
            .expect("ERROR: FAILED TO PERSIST COMMUNITIES");

        for (i, community) in communities.iter().enumerate() {
            for (j, vertex) in community.iter().enumerate() {
                if j == 0 {
                    write!(file, "{}", vertex)
                        .expect("ERROR: FAILED TO WRITE ON COMMUNITY PERSISTENCE");
                    continue;
                }
                write!(file, " {}", vertex)
                    .expect("ERROR: FAILED TO WRITE ON COMMUNITY PERSISTENCE");
            }

            if i != communities.len() - 1 {
                writeln!(file).expect("ERROR: FAILED TO WRITE LINE ON COMMUNITY PERSISTENCE");
            }
        }
    }
}
