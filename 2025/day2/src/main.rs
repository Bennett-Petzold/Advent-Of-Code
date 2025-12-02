use std::{num::ParseIntError, ops::RangeInclusive};

use advent_rust_lib::read::input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IDRange {
    start: u64,
    // Inclusive
    end: u64,
}

impl IDRange {
    pub fn find(line: &str) -> impl Iterator<Item = Result<Self, ParseIntError>> {
        line.split(",").flat_map(|pair| {
            pair.split_once('-').map(|(start, end)| {
                Ok(Self {
                    start: start.parse()?,
                    end: end.parse()?,
                })
            })
        })
    }

    pub fn ids(&self) -> RangeInclusive<u64> {
        self.start..=self.end
    }
}

fn is_single_repeated(id: u64) -> bool {
    let num_digits = ((id as f32).log10() as u32) + 1;
    // Splits digits evenly in half
    let div = 10_u64.pow(num_digits / 2);

    // Div gets top, mod gets bottom
    (id / div) == (id % div)
}

// Keeping as digits instead of string digit transforms should be cheaper.
fn is_at_least_one_repeated(id: u64) -> bool {
    let num_digits = ((id as f32).log10() as u32) + 1;

    // Moving digit split
    (1..=(num_digits / 2))
        // Fully repeated sequences require even division
        .filter(|cut| num_digits.is_multiple_of(*cut))
        .any(|cut| {
            let div = 10_u64.pow(cut);

            // Check that all N slices of the number have the sequence
            let mut rolling = id;
            let val = id % div;
            (1..(num_digits / cut)).all(|_rep| {
                rolling /= div;
                (rolling % div) == val
            })
        })
}

fn part1<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input
        .flat_map(|range| range.ids())
        .filter(|id| is_single_repeated(*id))
        .sum()
}

fn part2<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input
        .flat_map(|range| range.ids())
        .filter(|id| is_at_least_one_repeated(*id))
        .sum()
}

// About 149 ms execution on my machine.
fn main() {
    let id_ranges = {
        let input_line = input().next().unwrap();
        IDRange::find(&input_line)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };

    println!("Part 1: {}", part1(id_ranges.iter().copied()));
    println!("Part 2: {}", part2(id_ranges.into_iter()));
}
