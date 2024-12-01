use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use day3::EngineEntries;

fn main() {
    let entries = EngineEntries::from_iter(
        BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|x| x.unwrap()),
    );

    let part1 = entries.part_numbers().sum::<u32>();
    println!("{part1}");

    let part2 = entries.gears().into_iter().sum::<u32>();
    println!("{part2}");
}
