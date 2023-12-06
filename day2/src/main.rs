use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use day2::{Color, Game};

fn main() {
    let conditions = vec![(Color::Red, 12), (Color::Green, 13), (Color::Blue, 14)];

    let games = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Game::from_str(&line.unwrap()))
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap();

    let part1: u32 = games
        .iter()
        .filter(|game| game.mins_within(&conditions))
        .map(|game| *game.id())
        .sum();
    println!("{part1}");

    let part2: u32 = games
        .iter()
        .map(|game| game.mins())
        .map(|out_vec| out_vec.into_iter().reduce(|acc, x| acc * x).unwrap())
        .sum();
    println!("{part2}");
}
