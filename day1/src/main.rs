use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use day1::{digit, text_to_digit};

fn main() {
    let part1: u32 = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap())
        .map(digit)
        .collect::<Option<Vec<_>>>()
        .unwrap()
        .into_iter()
        .sum();
    println!("{part1}");

    let part2: u32 = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap())
        .map(text_to_digit)
        .map(digit)
        .collect::<Option<Vec<_>>>()
        .unwrap()
        .into_iter()
        .sum();
    println!("{part2}");

    /*
    let part2: Vec<_> = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| line.unwrap())
        .map(|x| {
            (
                x.clone(),
                text_to_digit(x.clone()),
                digit(text_to_digit(x.clone())),
            )
        })
        .collect();
    //.map(digit)
    println!("{:#?}", part2);
    */
}
