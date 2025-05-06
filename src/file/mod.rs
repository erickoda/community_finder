use std::{fs, str::FromStr};

pub struct File;

impl File {
    pub fn read<T>(path: impl Into<String>) -> Vec<[T; 2]>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let file_as_string =
            fs::read_to_string(path.into()).expect("FILE: INCORRECT PATH OR FILE INEXISTENT");

        let data: Vec<[T; 2]> = file_as_string
            .lines()
            .filter_map(|line| {
                let mut nums = line.split_whitespace().filter_map(|s| s.parse::<T>().ok());
                match (nums.next(), nums.next()) {
                    (Some(a), Some(b)) => Some([a, b]),
                    _ => None,
                }
            })
            .collect();

        data
    }
}
