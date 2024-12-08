use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
};

/// Return input lines (given as command arguments)
pub fn input() -> impl Iterator<Item = String> {
    BufReader::new(args().skip(1).map(|arg| File::open(arg)).unwrap())
        .lines()
        .map(|line| line.unwrap())
}
