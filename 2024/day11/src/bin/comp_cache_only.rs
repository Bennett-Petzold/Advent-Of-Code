use std::{collections::HashMap, env::args, num::ParseIntError};

use advent_rust_lib::read::filtered_input;

fn main() {
    let stones = StoneCollection::from_line(filtered_input(&[1]).next().unwrap()).unwrap();
    let num_iter: u64 = str::parse(&args().nth(2).unwrap()).unwrap();

    part_1(stones.clone(), num_iter);
}

fn part_1(mut stones: StoneCollection, num_iter: u64) {
    for _ in 0..num_iter {
        stones.step();
    }

    println!("{}", stones.len());
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
    counts: Vec<(u64, u64)>,
}

impl StoneCollection {
    pub fn from_line<S: AsRef<str>>(line: S) -> Result<Self, ParseIntError> {
        #[allow(clippy::trim_split_whitespace)]
        let counts: Vec<_> = line
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
        self.counts.iter().map(|(_, count)| count).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn condense(&mut self) {
        // Non-empty array
        if !self.counts.is_empty() {
            self.counts.sort_unstable_by_key(|(idx, _)| *idx);
            let cur_len = self.counts.len();
            let mut counts_iter =
                std::mem::replace(&mut self.counts, Vec::with_capacity(cur_len)).into_iter();

            self.counts
                .push(counts_iter.next().expect("counts had at least one element"));

            for (idx, count) in counts_iter {
                let (last_idx, last_count) = self
                    .counts
                    .last_mut()
                    .expect("counts must have at least one element");

                if idx == *last_idx {
                    *last_count += count;
                } else {
                    self.counts.push((idx, count));
                }
            }
        }
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
        let mut leftovers = Vec::new();

        for (idx, count) in &mut self.counts[..] {
            let children = self
                .comp_cache
                .entry(*idx)
                .or_insert_with(|| Self::gen_children(*idx));

            match children {
                ChildStones::One(child_idx) => *idx = *child_idx,
                ChildStones::Two([child_idx_0, child_idx_1]) => {
                    *idx = *child_idx_0;
                    leftovers.push((*child_idx_1, *count));
                }
            }
        }

        self.counts.append(&mut leftovers);
        self.condense();
    }

    /// Return length after applying a final step.
    pub fn final_step(&self) -> u64 {
        let mut sum = 0;

        for (idx, count) in &self.counts[..] {
            if let Some(children) = self.comp_cache.get(idx) {
                match children {
                    ChildStones::One(_) => sum += count,
                    ChildStones::Two(_) => {
                        sum += count * 2;
                    }
                }
            } else if (Self::num_digits(*idx) % 2) == 0 {
                sum += count * 2;
            } else {
                sum += count;
            }
        }

        sum
    }
}
