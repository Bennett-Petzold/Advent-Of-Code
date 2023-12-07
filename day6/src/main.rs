use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use day6::{parse_races, parse_single_race};

fn main() {
    let input: Vec<_> = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap())
        .collect();
    let races = parse_races(&input.clone().try_into().unwrap()).unwrap();

    let part1: u64 = races.iter().map(|r| r.num_ways_to_win()).product();
    println!("{part1}");

    let part2: u64 = parse_single_race(&input.clone().try_into().unwrap())
        .unwrap()
        .num_ways_to_win();
    println!("{part2}");
}
