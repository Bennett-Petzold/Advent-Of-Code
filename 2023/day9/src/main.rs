use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use day9::Sequence;

fn main() {
    let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Sequence::<i64>::from_str(&line.unwrap()).unwrap())
        .collect();

    let part1: i64 = seq.iter().map(|s| s.next()).sum();
    println!("{part1}");

    let part2: i64 = seq.iter().map(|s| s.prev()).sum();
    println!("{part2}");
}
