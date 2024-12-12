use std::{env::args, num::ParseIntError};

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

// Implements eq and sort based on value
#[derive(Debug, Clone, Copy)]
pub struct Stone {
    pub value: u64,
    // Sorted idxes
    pub count: u64,
}

impl Stone {
    pub fn new(value: u64) -> Self {
        Self { value, count: 1 }
    }

    /// Only perform this if self == rhs.
    pub fn merge(&mut self, rhs: Self) {
        self.count += rhs.count;
    }
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
pub struct StoneCollection {
    arr: Vec<Stone>,
}

impl StoneCollection {
    pub fn from_line<S: AsRef<str>>(line: S) -> Result<Self, ParseIntError> {
        #[allow(clippy::trim_split_whitespace)]
        let arr = line
            .as_ref()
            .trim()
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<u64>, _>>()?
            .into_iter()
            .map(Stone::new)
            .collect();

        let mut this = Self { arr };
        this.condense();
        Ok(this)
    }

    pub fn len(&self) -> u64 {
        self.arr.iter().map(|stone| stone.count).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.arr.is_empty()
    }

    pub fn condense(&mut self) {
        // Non-empty array
        if !self.arr.is_empty() {
            self.arr.sort_unstable_by_key(|x| x.value);
            let cur_len = self.arr.len();
            let mut arr_iter =
                std::mem::replace(&mut self.arr, Vec::with_capacity(cur_len)).into_iter();

            self.arr
                .push(arr_iter.next().expect("arr had at least one element"));

            for stone in arr_iter {
                let last_val = self
                    .arr
                    .last_mut()
                    .expect("arr must have at least one element");

                if stone.value == last_val.value {
                    last_val.merge(stone);
                } else {
                    self.arr.push(stone);
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
    fn split_digits_in_half(val: u64) -> (u64, u64) {
        let half = (val.ilog10() + 1) / 2;
        let divisor = 10_u64.pow(half);

        (val / divisor, val % divisor)
    }

    pub fn step(&mut self) {
        let mut new_entries = Vec::new();

        let start_idx = {
            if let Some(first_stone) = self.arr.first_mut() {
                if first_stone.value == 0 {
                    first_stone.value = 1;
                    1
                } else {
                    0
                }
            } else {
                0
            }
        };

        for stone in self.arr[start_idx..].iter_mut() {
            if (Self::num_digits(stone.value) % 2) == 0 {
                // Copy out to another stone
                let mut other_stone = *stone;

                // Split the value in half
                (stone.value, other_stone.value) = Self::split_digits_in_half(stone.value);

                // Store the new stone for later insert
                new_entries.push(other_stone);
            } else {
                stone.value *= 2024;
            }
        }

        // Push in new elements and remove dups
        self.arr.append(&mut new_entries);
        self.condense();
    }
}
