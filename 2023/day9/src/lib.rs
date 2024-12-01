#![cfg_attr(feature = "simd", feature(portable_simd))]
use std::{fmt::Debug, iter::Sum, ops::Deref, str::FromStr};

use itertools::Itertools;
use num::{Integer, Signed};

#[cfg(feature = "simd")]
use std::simd::{LaneCount, Simd, SimdElement, SimdInt, SupportedLaneCount};

#[cfg(feature = "simd")]
use std::ops::{Not, SubAssign};

#[derive(Debug, Clone)]
pub struct Sequence<T: Integer>(Vec<T>);

impl<T: Integer> Deref for Sequence<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Integer + FromStr> FromStr for Sequence<T> {
    type Err = <T as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_whitespace()
                .map(|part| part.parse())
                .try_collect()?,
        ))
    }
}

impl<T> Sequence<T>
where
    T: Integer + Sum + Signed + Copy,
{
    fn lower_sequence(seq: &[T]) -> impl Iterator<Item = T> + '_ {
        seq.iter()
            .zip(seq.iter().skip(1))
            .map(|(first, second)| *second - *first)
    }

    pub fn next(&self) -> T {
        let mut ends = Vec::with_capacity(self.0.len());

        let mut cur_seq = self.0.clone();
        while cur_seq.iter().any(|x| *x != T::zero()) {
            ends.push(*cur_seq.last().unwrap());
            cur_seq = Self::lower_sequence(&cur_seq).collect();
        }

        ends.into_iter().sum()
    }

    pub fn prev(&self) -> T {
        let mut starts = Vec::with_capacity(self.0.len());

        let mut cur_seq = self.0.clone();
        while cur_seq.iter().any(|x| *x != T::zero()) {
            starts.push(cur_seq[0]);
            cur_seq = Self::lower_sequence(&cur_seq).collect();
        }

        starts
            .into_iter()
            .rev()
            .reduce(|acc, x| x - acc)
            .unwrap_or(T::zero())
    }
}

#[cfg(feature = "simd")]
impl<T> Sequence<T>
where
    T: Integer + Signed + Sum + Clone + Copy + SimdElement,
    usize: TryFrom<T>,
{
    /// Right rotate and subtract to progress the sequence
    ///
    ///```ignore
    ///   [1] | [0]
    ///   0 3 | 6 9
    /// -     | x 6
    /// = 0 3 | x 3
    ///   [0][0] = [0][0]_prev - [1][1] = 6 - 3 = 3
    ///   [1] | [0]
    ///   0 3 | 3 3
    /// - x 0 |
    /// = x 3 | 3 3
    /// ```
    fn lower_sequence_simd<const N: usize>(seq: &mut [Simd<T, N>])
    where
        LaneCount<N>: SupportedLaneCount,
        Simd<T, N>: SubAssign,
    {
        let last_idx = seq.len() - 1;

        // For >0, need to shift in from the lower vector
        for idx in 0..last_idx {
            let hold = seq[idx].as_array()[0];
            let shift_copy = seq[idx].rotate_lanes_right::<1>();
            seq[idx] -= shift_copy;
            seq[idx].as_mut_array()[0] = hold - seq[idx + 1].as_array()[N - 1];
        }

        // For the last (lowest) entry, copy to no-op next cycle
        let shift_copy = seq[last_idx].rotate_lanes_right::<1>();
        seq[last_idx] -= shift_copy;
        let last_arr = seq[last_idx].as_mut_array();
        last_arr[0] = last_arr[1];
    }

    /// Turn an array into a reversed vector of zero-padded SIMD
    ///
    /// Returns: (address of first zero-pad, Reverse-Order SIMD Vector)
    fn to_simd_vec<const N: usize>(arr: &[T]) -> (usize, Vec<Simd<T, N>>)
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let start_invalid = arr.len() % N;

        let input_vec: Vec<_> = (0..start_invalid)
            .map(|_| T::zero())
            .chain(arr.iter().cloned())
            .collect();

        (
            start_invalid,
            input_vec
                .chunks_exact(N)
                .map(|chunk| Simd::from_slice(chunk))
                .rev()
                .collect(),
        )
    }

    /// Next implementation over SIMD vectors
    pub fn next_simd<const N: usize>(&self) -> T
    where
        LaneCount<N>: SupportedLaneCount,
        Simd<T, N>: SubAssign + SimdInt<Scalar = T> + Not<Output = Simd<T, N>>,
    {
        let (mut num_invalid, mut sections) = Self::to_simd_vec(&self.0);
        let mut ends: Vec<_> = Vec::with_capacity(sections.len() * N);

        // Loop while there are nonzero values
        while sections.iter().any(|sec| sec.reduce_or() != T::zero()) {
            ends.push(*sections[0].as_array().last().unwrap());
            Self::lower_sequence_simd(&mut sections);

            num_invalid += 1;
            if num_invalid == N {
                let _ = sections.pop();
                num_invalid = 0;
            }
        }

        ends.into_iter().sum()
    }

    /// Prev implementation over SIMD vectors
    pub fn prev_simd<const N: usize>(&self) -> T
    where
        LaneCount<N>: SupportedLaneCount,
        Simd<T, N>: SubAssign + SimdInt<Scalar = T> + Not<Output = Simd<T, N>>,
    {
        let (mut num_invalid, mut sections) = Self::to_simd_vec(&self.0);
        let mut starts: Vec<_> = Vec::with_capacity(sections.len() * N);

        // Loop while there are nonzero values
        while sections.iter().any(|sec| sec.reduce_or() != T::zero()) {
            starts.push(sections.last().unwrap().as_array()[num_invalid]);
            Self::lower_sequence_simd(&mut sections);

            num_invalid += 1;
            if num_invalid == N {
                let _ = sections.pop();
                num_invalid = 0;
            }
        }

        starts
            .into_iter()
            .rev()
            .reduce(|acc, x| x - acc)
            .unwrap_or(T::zero())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        num::ParseIntError,
    };

    use super::*;

    #[test]
    fn part1() {
        let input = [
            "0   3   6   9  12  15",
            "1   3   6  10  15  21",
            "10  13  16  21  30  45",
        ];

        let res = input
            .into_iter()
            .map(Sequence::<i64>::from_str)
            .collect::<Result<Vec<_>, ParseIntError>>()
            .unwrap()
            .into_iter()
            .map(|s| s.next())
            .sum::<i64>();
        assert_eq!(res, 114);
    }

    #[test]
    fn part2() {
        let input = [
            "0   3   6   9  12  15",
            "1   3   6  10  15  21",
            "10  13  16  21  30  45",
        ];

        let res = input
            .into_iter()
            .map(Sequence::<i64>::from_str)
            .collect::<Result<Vec<_>, ParseIntError>>()
            .unwrap()
            .into_iter()
            .map(|s| s.prev())
            .sum::<i64>();
        assert_eq!(res, 2);
    }

    #[test]
    fn part1_i32() {
        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i32>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(seq.iter().map(|s| s.next()).sum::<i32>(), 1898776583);
    }

    #[test]
    fn part1_i64() {
        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i64>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(seq.iter().map(|s| s.next()).sum::<i64>(), 1898776583);
    }

    #[test]
    fn part2_i32() {
        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i32>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(seq.iter().map(|s| s.prev()).sum::<i32>(), 1100);
    }

    #[test]
    fn part2_i64() {
        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i64>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(seq.iter().map(|s| s.prev()).sum::<i64>(), 1100);
    }

    #[cfg(feature = "simd")]
    #[test]
    fn part1_i32_simd() {
        let input = [
            "0   3   6   9  12  15",
            "1   3   6  10  15  21",
            "10  13  16  21  30  45",
        ];

        let res = input
            .into_iter()
            .map(|line| Sequence::<i32>::from_str(line).unwrap())
            .map(|s| s.next_simd::<2>())
            .sum::<i32>();
        assert_eq!(res, 114);

        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i32>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(
            seq.iter().map(|s| s.next_simd::<2>()).sum::<i32>(),
            1898776583
        );
    }

    #[cfg(feature = "simd")]
    #[test]
    fn part2_i32_simd() {
        let input = [
            "0   3   6   9  12  15",
            "1   3   6  10  15  21",
            "10  13  16  21  30  45",
        ];

        let res = input
            .into_iter()
            .map(|line| Sequence::<i32>::from_str(line).unwrap())
            .map(|s| s.prev_simd::<2>())
            .sum::<i32>();
        assert_eq!(res, 2);

        let seq: Vec<_> = BufReader::new(File::open("input").unwrap())
            .lines()
            .map(|line| Sequence::<i32>::from_str(&line.unwrap()).unwrap())
            .collect();
        assert_eq!(seq.iter().map(|s| s.prev_simd::<2>()).sum::<i32>(), 1100);
    }
}
