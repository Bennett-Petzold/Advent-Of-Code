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
