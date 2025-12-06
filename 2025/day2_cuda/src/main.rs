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

// About 271 ms execution on my machine.
fn main() {
    let all_ids = {
        let input_line = input().next().unwrap();
        IDRange::find(&input_line)
            .flat_map(|range| range.unwrap().ids())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    let init = InitStream::init(&all_ids).unwrap();
    let part1 = Task::part1(&init).unwrap();
    let part2 = Task::part2(&init).unwrap();

    println!("Part 1: {}", part1.resolve().unwrap());
    println!("Part 2: {}", part2.resolve().unwrap());
}
