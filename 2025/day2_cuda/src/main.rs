use std::{num::ParseIntError, ops::RangeInclusive};

use advent_rust_lib::read::input;

use crate::cuda::{InitStream, Task};

mod cuda;

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

fn part1<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    let input_expanded: Vec<_> = input.flat_map(|range| range.ids()).collect();
    let init = InitStream::init(&input_expanded).unwrap();
    Task::part1(&init).unwrap().resolve().unwrap()
}

fn part2<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    let input_expanded: Vec<_> = input.flat_map(|range| range.ids()).collect();
    let init = InitStream::init(&input_expanded).unwrap();
    Task::part2(&init).unwrap().resolve().unwrap()
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
