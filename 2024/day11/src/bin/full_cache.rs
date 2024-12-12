use std::{collections::HashMap, env::args, num::ParseIntError};

use advent_rust_lib::read::filtered_input;

fn main() {
    let stones = StoneCollection::from_line(filtered_input(&[1]).next().unwrap()).unwrap();
    let num_iter: u64 = str::parse(&args().nth(2).unwrap()).unwrap();

    part_1(stones.clone(), num_iter);
}

fn part_1(mut stones: StoneCollection, num_iter: u64) {
    for _ in 0..(num_iter.saturating_sub(1)) {
        stones.step();
    }

    if num_iter > 0 {
        println!("{}", stones.final_step());
    } else {
        println!("{}", stones.len());
    }
}

// -------------------------------------------------- //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChildStones {
    One(u64),
    Two([u64; 2]),
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
pub struct StoneCollection {
    comp_cache: HashMap<u64, ChildStones>,
    counts: HashMap<u64, u64>,
}

impl StoneCollection {
    pub fn from_line<S: AsRef<str>>(line: S) -> Result<Self, ParseIntError> {
        #[allow(clippy::trim_split_whitespace)]
        let counts: HashMap<_, _> = line
            .as_ref()
            .trim()
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<u64>, _>>()?
            .into_iter()
            .map(|val| (val, 1))
            .collect();

        // Initialize with the zero special case
        let comp_cache: HashMap<_, _> = [(0, ChildStones::One(1))].into();

        Ok(Self { comp_cache, counts })
    }

    pub fn gen_children(value: u64) -> ChildStones {
        let num_digits = Self::num_digits(value);
        if (num_digits % 2) == 0 {
            let (child0, child1) = Self::split_digits_at(value, num_digits / 2);

            // Store the new stone for later insert
            ChildStones::Two([child0, child1])
        } else {
            ChildStones::One(value * 2024)
        }
    }

    pub fn len(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn num_digits(val: u64) -> u32 {
        val.ilog10() + 1
    }

    /// Return (upper half of digits, lower half of digits)
    ///
    /// Uses base 10 for digits.
    fn split_digits_at(val: u64, split_point: u32) -> (u64, u64) {
        let divisor = 10_u64.pow(split_point);

        (val / divisor, val % divisor)
    }

    pub fn step(&mut self) {
        let prev_counts: Vec<_> = self
            .counts
            .iter()
            .map(|(x, y)| (*x, *y))
            .filter(|(_, count)| *count != 0)
            .collect();
        self.counts.values_mut().for_each(|val| *val = 0);

        for (idx, count) in prev_counts {
            let children = self
                .comp_cache
                .entry(idx)
                .or_insert_with(|| Self::gen_children(idx));

            match children {
                ChildStones::One(child_idx) => *self.counts.entry(*child_idx).or_insert(0) += count,
                ChildStones::Two([child_idx_0, child_idx_1]) => {
                    *self.counts.entry(*child_idx_0).or_insert(0) += count;
                    *self.counts.entry(*child_idx_1).or_insert(0) += count;
                }
            }
        }
    }

    /// Return length after applying a final step.
    pub fn final_step(&self) -> u64 {
        let mut sum = 0;

        let prev_counts: Vec<_> = self
            .counts
            .iter()
            .map(|(x, y)| (*x, *y))
            .filter(|(_, count)| *count != 0)
            .collect();

        for (idx, count) in prev_counts {
            if let Some(children) = self.comp_cache.get(&idx) {
                match children {
                    ChildStones::One(_) => sum += count,
                    ChildStones::Two(_) => {
                        sum += count * 2;
                    }
                }
            } else if (Self::num_digits(idx) % 2) == 0 {
                sum += count * 2;
            } else {
                sum += count;
            }
        }

        sum
    }
}
