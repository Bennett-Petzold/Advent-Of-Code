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
#[derive(Debug, Clone)]
pub struct Stone {
    pub value: u64,
    // Sorted idxes
    idxes: Vec<usize>,
}

impl Stone {
    pub fn new(value: u64, idx: usize) -> Self {
        Self {
            value,
            idxes: vec![idx],
        }
    }

    pub fn idxes(&self) -> &[usize] {
        &self.idxes
    }

    pub fn idxes_mut(&mut self) -> &mut [usize] {
        &mut self.idxes
    }

    /// Only perform this if self == rhs.
    pub fn merge(&mut self, mut rhs: Self) {
        self.idxes.append(&mut rhs.idxes);
        self.idxes.sort_unstable();
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
            .enumerate()
            .map(|(idx, num)| Stone::new(num, idx))
            .collect();

        let mut this = Self { arr };
        this.condense();
        Ok(this)
    }

    pub fn len(&self) -> usize {
        self.arr.iter().map(|stone| stone.idxes().len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.arr.iter().all(|stone| stone.idxes().is_empty())
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
        let mut new_arr: Vec<Stone> = Vec::with_capacity(self.arr.len() * 2);

        let insert_new = |stone: Stone, target_arr: &mut Vec<Stone>| match target_arr
            .binary_search_by_key(&stone.value, |x| x.value)
        {
            Ok(idx) => target_arr[idx].merge(stone),
            Err(idx) => target_arr.insert(idx, stone),
        };

        for mut stone in std::mem::take(&mut self.arr) {
            if stone.value == 0 {
                stone.value = 1;
            } else if (Self::num_digits(stone.value) % 2) == 0 {
                stone.idxes().iter().rev().for_each(|idx| {
                    new_arr.iter_mut().for_each(|new_stone| {
                        // Modify all entries above the partition point
                        let partition_point = new_stone
                            .idxes()
                            .partition_point(|inner_idx| *inner_idx > *idx);

                        new_stone.idxes_mut()[partition_point..]
                            .iter_mut()
                            .for_each(|inner_idx| *inner_idx += 1);
                    })
                });

                // Split into following stones
                let mut other_stone = stone.clone();
                other_stone.idxes_mut().iter_mut().for_each(|idx| *idx += 1);

                // Split the value in half
                (stone.value, other_stone.value) = Self::split_digits_in_half(stone.value);

                // Insert the new stone
                insert_new(other_stone, &mut new_arr);
            } else {
                stone.value *= 2024;
            }

            insert_new(stone, &mut new_arr);
        }

        // Complete the shift
        self.arr = new_arr;
    }
}
