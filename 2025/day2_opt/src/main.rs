#![feature(portable_simd)]

use std::{
    mem,
    num::ParseIntError,
    ops::RangeInclusive,
    simd::{
        Mask, Simd, StdFloat,
        cmp::{SimdPartialEq, SimdPartialOrd},
        num::{SimdFloat, SimdInt, SimdUint},
    },
};

use advent_rust_lib::read::input;

// Assuming AVX2 for optimization.
pub const SIMD_VEC_BIT_SIZE: usize = 256;
// Efficient packing of u64 in the vec.
pub const SIMD_VEC_ELEMENTS: usize = 256 / (u64::BITS as usize);
type IDSimd = Simd<u64, SIMD_VEC_ELEMENTS>;

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

    pub const fn ids(&self) -> RangeInclusive<u64> {
        self.start..=self.end
    }

    #[inline]
    /// Sums the results of count_func over all elements in range.
    ///
    /// FSimd needs to count up the number of passing funcs.
    /// FSimd must not count elements that are 1.
    pub fn simd_sums<FSimd>(&self, count_func: FSimd) -> u64
    where
        FSimd: Fn(IDSimd) -> u64,
    {
        const INIT: IDSimd = IDSimd::from_array([0, 1, 2, 3]);
        const ADVANCE: IDSimd = IDSimd::splat(SIMD_VEC_ELEMENTS as u64);

        let mut remaining_len = (self.end - self.start) + 1;
        let mut acting_array = Simd::splat(self.start) + INIT;
        let mut sum = 0;

        loop {
            if remaining_len < (SIMD_VEC_ELEMENTS as u64) {
                for loc in &mut acting_array.as_mut_array()[remaining_len as usize..] {
                    *loc = 1;
                }
            }

            sum += count_func(acting_array);

            // Loops ends when no elements are left
            remaining_len = remaining_len.saturating_sub(SIMD_VEC_ELEMENTS as u64);
            if remaining_len == 0 {
                break;
            }
            // All elements step up by the given size
            acting_array += ADVANCE;
        }

        sum
    }

    #[inline]
    /// Sums the results of count_func over all elements in range.
    ///
    /// FSimd needs to count up the number of passing funcs.
    /// FSimd must not count elements that are 1.
    pub fn simd_sums_part1(&self) -> u64 {
        const INIT: IDSimd = IDSimd::from_array([0, 1, 2, 3]);
        const ADVANCE: IDSimd = IDSimd::splat(SIMD_VEC_ELEMENTS as u64);

        let mut remaining_len = (self.end - self.start) + 1;
        let mut acting_array = Simd::splat(self.start) + INIT;
        let mut sum = 0;

        loop {
            if remaining_len < (SIMD_VEC_ELEMENTS as u64) {
                for loc in &mut acting_array.as_mut_array()[remaining_len as usize..] {
                    *loc = 1;
                }
            }

            // Sum function
            {
                let first_num_digits = (acting_array.as_array()[0] as f32).log10() as u32;
                let all_num_digits = (acting_array.cast::<f32>()).log10().cast::<u32>();

                let base_div = Simd::splat(10_u64.pow((first_num_digits + 1) / 2));
                let greater = Simd::splat(first_num_digits).simd_gt(all_num_digits);
                let div_mult = greater
                    .cast::<i64>()
                    .select(Simd::splat(10), Simd::splat(1));
                let div = base_div * div_mult;

                // Creates a bitmask
                let passing = (acting_array / div).simd_eq(acting_array % div);
                // Zero all non-passing values
                // Transmute is necessary to keep the all-1 bit fields.
                // SAFETY: SIMD types have the same size and are numbers.
                let masked_ones =
                    unsafe { mem::transmute::<Mask<i64, SIMD_VEC_ELEMENTS>, IDSimd>(passing) }
                        & acting_array;

                // Adds all true results
                sum += masked_ones.reduce_sum()
            }

            // Loops ends when no elements are left
            remaining_len = remaining_len.saturating_sub(SIMD_VEC_ELEMENTS as u64);
            if remaining_len == 0 {
                break;
            }
            // All elements step up by the given size
            acting_array += ADVANCE;
        }

        sum
    }
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

/*
fn part1<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input
        .map(|range| {
            range.simd_sums(|vec| {
                const ONE_VEC: Simd<u32, SIMD_VEC_ELEMENTS> = Simd::splat(1);

                let num_digits = (vec.cast::<f32>()).log10().cast::<u32>() + ONE_VEC;

                // Nice splat if all entries are equal, unfortunate per-element otherwise
                let div = if num_digits.reduce_and() == num_digits.as_array()[0] {
                    Simd::splat(10_u64.pow((num_digits.as_array()[0] / 2) as u32))
                } else {
                    Simd::from_array(num_digits.as_array().map(|entry| 10_u64.pow(entry / 2)))
                };

                // Creates a bitmask
                let passing = (vec / div).simd_eq(vec % div);
                // Zero all non-passing values
                // Transmute is necessary to keep the all-1 bit fields.
                // SAFETY: SIMD types have the same size and are numbers.
                let masked_ones = unsafe {
                    mem::transmute::<Simd<i64, SIMD_VEC_ELEMENTS>, IDSimd>(passing.to_int())
                } & vec;

                // Adds all true results
                masked_ones.reduce_sum()
            })
        })
        .sum()
}
*/

fn part1<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input.map(|range| range.simd_sums_part1()).sum()
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

// About _ ms execution on my machine.
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
