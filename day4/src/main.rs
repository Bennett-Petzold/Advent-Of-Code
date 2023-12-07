use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use day4::Card;

fn main() {
    let cards = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Card::from_str(&line.unwrap()))
        .collect::<anyhow::Result<Vec<Card>>>()
        .unwrap();

    let part1: usize = cards
        .clone()
        .into_iter()
        .map(|card| card.match_points())
        .sum();
    println!("{part1}");

    let part2 = Card::total_scratchcards(cards);
    println!("{part2}");
}
