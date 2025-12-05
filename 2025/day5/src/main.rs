use std::{cmp::max, num::ParseIntError};

use advent_rust_lib::read::input;
use derive_getters::Getters;

/// Inclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Getters)]
pub struct FreshRange {
    start: u64,
    // Always >= start
    end: u64,
}

impl PartialOrd for FreshRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FreshRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start).then(self.end.cmp(&other.end))
    }
}

impl FreshRange {
    /// Returns None on a blank line.
    fn from_line(s: &str) -> Option<Result<Self, ParseIntError>> {
        let (start, end) = s.split_once("-")?;

        match start.parse() {
            Ok(start) => match end.parse() {
                Ok(end) => Some(Ok(Self { start, end })),
                Err(e) => Some(Err(e)),
            },
            Err(e) => Some(Err(e)),
        }
    }

    /// Combines the two ranges if they overlap.
    fn union(self, other: Self) -> Option<Self> {
        let (before, after) = if self.start <= other.start {
            (self, other)
        } else {
            (other, self)
        };

        (after.start <= before.end).then(|| {
            let max_end = max(self.end, other.end);
            Self {
                start: self.start,
                end: max_end,
            }
        })
    }

    fn contains(&self, ingredient: u64) -> bool {
        (self.start <= ingredient) && (self.end >= ingredient)
    }

    fn len(&self) -> u64 {
        (self.end - self.start) + 1
    }
}

fn part1<R, I>(ranges: R, ingredients: I) -> u64
where
    R: IntoIterator<Item = FreshRange>,
    I: IntoIterator<Item = u64>,
{
    let mut ranges = ranges.into_iter();

    let mut count = 0;

    if let Some(mut range) = ranges.next() {
        for ingredient in ingredients.into_iter() {
            if range.contains(ingredient) {
                count += 1
            } else if *range.start() <= ingredient {
                loop {
                    if let Some(next_range) = ranges.next() {
                        range = next_range;
                        if *range.start() > ingredient {
                            break;
                        } else if range.contains(ingredient) {
                            count += 1;
                            break;
                        }
                    } else {
                        // All later ingredients can be ignored.
                        return count;
                    }
                }
            }
        }
    }

    count
}

/// Executes in about 940 microseconds on my machine.
fn main() {
    let mut input = input();

    // Sorted with smallest start first, all overlaps combined
    let fresh_ranges = {
        let mut fresh_ranges = input
            .by_ref()
            .map(|line| FreshRange::from_line(&line))
            .take_while(|range| range.is_some())
            .flatten()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        fresh_ranges.sort_unstable();

        let mut idx = 1;
        while idx < fresh_ranges.len() {
            if let Some(combined) = fresh_ranges[idx - 1].union(fresh_ranges[idx]) {
                fresh_ranges[idx - 1] = combined;
                let _removed_redundant = fresh_ranges.remove(idx);
            } else {
                idx += 1;
            }
        }

        fresh_ranges
    };

    // Sorted with smallest ingredient first
    let ingredients = {
        let mut ingredients = input
            .map(|line| line.parse())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        ingredients.sort_unstable();
        ingredients
    };

    println!(
        "Part 1: {}",
        part1(fresh_ranges.iter().cloned(), ingredients.iter().cloned())
    );
    println!(
        "Part 2: {}",
        fresh_ranges.iter().map(|range| range.len()).sum::<u64>()
    );
}
