use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use day7::{Hand, HandType, WildCardHandType};
use itertools::Itertools;

fn main() {
    let part1: usize = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Hand::<HandType>::from_str(&line.unwrap()).unwrap())
        .sorted()
        .enumerate()
        .map(|(idx, hand)| (idx + 1) * hand.bid())
        .sum();
    println!("{part1}");

    let part2: usize = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Hand::<WildCardHandType>::from_str(&line.unwrap()).unwrap())
        .sorted()
        .enumerate()
        .map(|(idx, hand)| (idx + 1) * hand.bid())
        .sum();
    println!("{part2}");
}
