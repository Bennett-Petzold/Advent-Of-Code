#![feature(portable_simd)]

use std::{
    array, mem,
    num::ParseIntError,
    ops::RangeInclusive,
    simd::{Mask, Simd, cmp::SimdPartialEq, num::SimdUint},
};

use advent_rust_lib::read::input;
use multiversion::{multiversion, target::selected_target};

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
}

#[inline]
/// Outer function that calls more efficient inners
pub fn simd_sums_part1(mut range: IDRange) -> u64 {
    macro_rules! simd_sums_inner {
        ($uint:ty, $int:ty) => {
            paste::paste! {
                #[inline]
                #[multiversion(targets = "simd")]
                fn [<simd_sums_inner_ $uint>](start: $uint, end: $uint, num_digits: u32) -> u64 {
                    const SIMD_VEC_ELEMENTS: usize =
                        selected_target!().suggested_simd_width::<$uint>().unwrap();
                    type IdSimd = Simd<$uint, SIMD_VEC_ELEMENTS>;

                    let init: IdSimd = Simd::from_array(array::from_fn(|idx| idx as $uint));
                    const ADVANCE: IdSimd = Simd::splat(SIMD_VEC_ELEMENTS as $uint);

                    const TEN: $uint = 10;

                    let mut remaining_len = (end - start) + 1;
                    let mut acting_array = Simd::splat(start) + init;
                    let mut sum = 0;

                    let div = Simd::splat(TEN.pow(num_digits / 2));

                    loop {
                        if remaining_len < (SIMD_VEC_ELEMENTS as $uint) {
                            for loc in &mut acting_array.as_mut_array()[remaining_len as usize..] {
                                *loc = 1;
                            }
                        }

                        // Sum function
                        {
                            // Creates a bitmask
                            let passing = (acting_array / div).simd_eq(acting_array % div);
                            // Zero all non-passing values
                            // Transmute is necessary to keep the all-1 bit fields.
                            // SAFETY: SIMD types have the same size and are numbers.
                            let masked_ones =
                                unsafe { mem::transmute::<Mask<$int, SIMD_VEC_ELEMENTS>, IdSimd>(passing) }
                                    & acting_array;

                            // Adds all true results
                            sum += masked_ones.cast::<u64>().reduce_sum()
                        }

                        // Loops ends when no elements are left
                        remaining_len = remaining_len.saturating_sub(SIMD_VEC_ELEMENTS as $uint);
                        if remaining_len == 0 {
                            break;
                        }
                        // All elements step up by the given size
                        acting_array += ADVANCE;
                    }

                    sum
                }
            }
        };
    }

    simd_sums_inner!(u64, i64);
    simd_sums_inner!(u32, i32);
    simd_sums_inner!(u16, i16);
    simd_sums_inner!(u8, i8);

    let mut remainder = None;

    // Makes sure entire range shares the number of digits
    let first_num_digits = ((range.start as f32).log10() as u8) + 1;
    let last_num_digits = ((range.end as f32).log10() as u8) + 1;

    if first_num_digits != last_num_digits {
        let split = 10_u64.pow(first_num_digits as u32);

        remainder = Some(IDRange {
            start: split,
            end: range.end,
        });
        range.end = split - 1;
    }

    let sum = match range.end {
        end if end <= (u8::MAX as u64) => {
            simd_sums_inner_u8(range.start as u8, end as u8, first_num_digits as u32)
        }
        end if end <= (u16::MAX as u64) => {
            simd_sums_inner_u16(range.start as u16, end as u16, first_num_digits as u32)
        }
        end if end <= (u32::MAX as u64) => {
            simd_sums_inner_u32(range.start as u32, end as u32, first_num_digits as u32)
        }
        // Presumed <= u64::MAX, undefined if this is false
        end => simd_sums_inner_u64(range.start, end, first_num_digits as u32),
    };

    sum + remainder.map_or(0, simd_sums_part1)
}

#[inline]
/// Outer function that calls more efficient inners
pub fn simd_sums_part2(mut range: IDRange) -> u64 {
    macro_rules! simd_sums_inner {
        ($uint:ty, $int:ty) => {
            paste::paste! {
                #[inline]
                #[multiversion(targets = "simd")]
                fn [<simd_sums_inner_ $uint>](start: $uint, end: $uint, num_digits: u32) -> u64 {
                    const SIMD_VEC_ELEMENTS: usize =
                        selected_target!().suggested_simd_width::<$uint>().unwrap();
                    type IdSimd = Simd<$uint, SIMD_VEC_ELEMENTS>;

                    let init: IdSimd = Simd::from_array(array::from_fn(|idx| idx as $uint));
                    const ADVANCE: IdSimd = Simd::splat(SIMD_VEC_ELEMENTS as $uint);

                    const TEN: $uint = 10;

                    let mut remaining_len = (end - start) + 1;
                    let mut acting_array = Simd::splat(start) + init;
                    let mut sum = 0;

                    let cuts: Vec<_> = (1..=(num_digits / 2))
                        .filter(|cut| num_digits.is_multiple_of(*cut))
                        .map(|cut| (TEN.pow(cut), num_digits / cut))
                        .collect();

                    loop {
                        if remaining_len < (SIMD_VEC_ELEMENTS as $uint) {
                            for loc in &mut acting_array.as_mut_array()[remaining_len as usize..] {
                                *loc = 1;
                            }
                        }

                        // Rotation checks if array is all equal elements
                        let mut passing = Mask::from_bitmask(0);
                        for (div, reps) in &cuts {
                            let div = Simd::splat(*div);
                            let val = acting_array % div;

                            let mut rolling_array = acting_array;
                            let mut inner_passing = Mask::from_bitmask(u64::MAX);

                            // Build inner passing bitmask
                            for _ in (1..*reps) {
                                rolling_array /= div;
                                // Removes any failed checks
                                inner_passing &= (rolling_array % div).simd_eq(val);
                            }
                            // Add any newly passing entries to bitmask
                            passing |= inner_passing;
                        }

                        // Zero all non-passing values
                        // Transmute is necessary to keep the all-1 bit fields.
                        // SAFETY: SIMD types have the same size and are numbers.
                        let masked_ones =
                            unsafe { mem::transmute::<Mask<$int, SIMD_VEC_ELEMENTS>, IdSimd>(passing) }
                                & acting_array;

                        // Adds all true results
                        sum += masked_ones.cast::<u64>().reduce_sum();

                        // Loops ends when no elements are left
                        remaining_len = remaining_len.saturating_sub(SIMD_VEC_ELEMENTS as $uint);
                        if remaining_len == 0 {
                            break;
                        }
                        // All elements step up by the given size
                        acting_array += ADVANCE;
                    }

                    sum
                }
            }
        };
    }

    simd_sums_inner!(u64, i64);
    simd_sums_inner!(u32, i32);
    simd_sums_inner!(u16, i16);
    simd_sums_inner!(u8, i8);

    let mut remainder = None;

    // Makes sure entire range shares the number of digits
    let first_num_digits = ((range.start as f32).log10() as u8) + 1;
    let last_num_digits = ((range.end as f32).log10() as u8) + 1;

    if first_num_digits != last_num_digits {
        let split = 10_u64.pow(first_num_digits as u32);

        remainder = Some(IDRange {
            start: split,
            end: range.end,
        });
        range.end = split - 1;
    }

    let sum = match range.end {
        end if end <= (u8::MAX as u64) => {
            simd_sums_inner_u8(range.start as u8, end as u8, first_num_digits as u32)
        }
        end if end <= (u16::MAX as u64) => {
            simd_sums_inner_u16(range.start as u16, end as u16, first_num_digits as u32)
        }
        end if end <= (u32::MAX as u64) => {
            simd_sums_inner_u32(range.start as u32, end as u32, first_num_digits as u32)
        }
        // Presumed <= u64::MAX, undefined if this is false
        end => simd_sums_inner_u64(range.start, end, first_num_digits as u32),
    };

    sum + remainder.map_or(0, simd_sums_part2)
}

fn part1<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input.map(simd_sums_part1).sum()
}

fn part2<IDs>(input: IDs) -> u64
where
    IDs: Iterator<Item = IDRange>,
{
    input.map(simd_sums_part2).sum()
}

// About 110 ms execution on my machine.
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
