use std::fs;

pub struct File;

impl File {
    pub fn read(path: impl Into<String>) -> Vec<[i32;2]> {
        let file_as_string = fs::read_to_string(path.into()).expect("FILE: INCORRECT PATH OR FILE INEXISTENT");

        let data: Vec<[i32; 2]> = file_as_string
        .lines()
        .filter_map(|line| {
            let nums: Vec<i32> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if nums.len() == 2 {
                Some([nums[0], nums[1]])
            } else {
                None
            }
        })
        .collect();

        data
    }
}
