use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use day8::{Directions, Mappings};

fn main() {
    let mut input = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap());

    let dirs = Directions::from_str(&input.next().unwrap()).unwrap();
    let maps = Mappings::from_lines(input.skip(1)).unwrap();

    let part1 = maps.walk(&dirs).unwrap();
    println!("{part1}");

    let part2 = maps.ghost_walk(&dirs).unwrap();
    println!("{part2}");
}
