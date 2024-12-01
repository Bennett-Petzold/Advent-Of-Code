use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use day5::Almanac;

fn main() {
    let input = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap());
    let almanac = Almanac::from_almanac_iter(input).unwrap();

    let part1 = almanac.locations().min().unwrap();
    println!("{part1}");

    let part2 = almanac.range_locations().flatten().min().unwrap();
    println!("{part2}");
}
