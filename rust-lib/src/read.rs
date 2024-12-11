use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
};

/// Return input lines (given as command arguments)
pub fn input() -> impl Iterator<Item = String> {
    args().skip(1).flat_map(|arg| {
        BufReader::new(File::open(arg).unwrap())
            .lines()
            .map(|line| line.unwrap())
    })
}

/// Return given input lines (given as command arguments)
///
/// * idxs: List of arg indicies to read
pub fn filtered_input(valid_idxs: &[usize]) -> impl Iterator<Item = String> + use<'_> {
    args()
        .enumerate()
        .skip(1)
        .filter(|(idx, _)| valid_idxs.contains(idx))
        .map(|(_, arg)| arg)
        .flat_map(|arg| {
            BufReader::new(File::open(arg).unwrap())
                .lines()
                .map(|line| line.unwrap())
        })
}
