use std::{env::args, num::ParseIntError, u8};

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

/// Return the number of base 10 digits
///
/// Max number of digits for u64 -> 20
/// 20 < u8::MAX (20 < 255)
fn num_digits(val: u64) -> u8 {
    let log = val.ilog10();
    debug_assert!(log < u8::MAX as u32);
    (log as u8) + 1
}

/// Return (upper half of digits, lower half of digits)
///
/// Uses base 10 for digits.
fn split_digits_at(val: u64, split_point: u8) -> (u64, u64) {
    let divisor = 10_u64.pow(split_point as u32);

    (val / divisor, val % divisor)
}

// -------------------------------------------------- //

// Implements eq and sort based on value
#[derive(Debug, Clone, Copy)]
pub struct Stone {
    pub value: u64,
    // Sorted idxes
    pub count: u64,
    pub even_digits: Option<u8>,
}

impl Stone {
    pub fn new(value: u64) -> Self {
        let even_digits = (value != 0)
            .then(|| {
                let num_digits = num_digits(value);
                ((num_digits % 2) == 0).then_some(num_digits)
            })
            .flatten();

        Self {
            value,
            count: 1,
            even_digits,
        }
    }

    /// Only perform this if self == rhs.
    pub fn merge(&mut self, rhs: Self) {
        self.count += rhs.count;
    }

    /// Multiplies value by `new_value` and updates even.
    pub fn update_value(&mut self, new_value: u64) {
        self.value *= new_value;
        let num_digits = num_digits(self.value);
        self.even_digits = ((num_digits % 2) == 0).then_some(num_digits);
    }

    /// Updates even digits.
    pub fn set_digits(&mut self) {
        let num_digits = num_digits(self.value);
        self.even_digits = ((num_digits % 2) == 0).then_some(num_digits);
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
            if let Some(digits) = stone.even_digits {
                let halved_digits = digits / 2;

                // Copy out to another stone
                let mut other_stone = *stone;

                // Split the value in half
                (stone.value, other_stone.value) = split_digits_at(stone.value, halved_digits);

                // Top part is never truncated, so its always halved digits
                stone.even_digits = ((halved_digits % 2) == 0).then_some(halved_digits);

                // Bottom part may be truncated, needs to recalc digits.
                if other_stone.value == 0 {
                    other_stone.even_digits = None;
                } else {
                    other_stone.set_digits()
                }

                // Store the new stone for later insert
                new_entries.push(other_stone);
            } else {
                stone.update_value(2024);
            }
        }

        // Push in new elements and remove dups
        self.arr.append(&mut new_entries);
        self.condense();
    }
}
