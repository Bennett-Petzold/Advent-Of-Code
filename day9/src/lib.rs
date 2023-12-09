use std::{ops::Deref, str::FromStr};

use itertools::Itertools;

#[derive(Debug)]
pub struct Sequence(Vec<i64>);

impl Deref for Sequence {
    type Target = Vec<i64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Sequence {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_whitespace()
                .map(|part| part.parse())
                .try_collect()?,
        ))
    }
}

impl Sequence {
    fn lower_sequence(seq: &[i64]) -> impl Iterator<Item = i64> + '_ {
        seq.iter()
            .zip(seq.iter().skip(1))
            .map(|(first, second)| second - first)
    }

    pub fn next(&self) -> Option<i64> {
        let mut ends = Vec::with_capacity(self.0.len());

        let mut cur_seq = self.0.clone();
        while cur_seq.iter().any(|x| x != &0) {
            ends.push(*cur_seq.last().unwrap());
            let next_seq: Vec<i64> = Self::lower_sequence(&cur_seq).collect();
            cur_seq = next_seq;
        }

        Some(ends.into_iter().sum())
    }

    pub fn prev(&self) -> Option<i64> {
        let mut starts = Vec::with_capacity(self.0.len());

        let mut cur_seq = self.0.clone();
        while cur_seq.iter().any(|x| x != &0) {
            starts.push(*cur_seq.first().unwrap());
            let next_seq: Vec<i64> = Self::lower_sequence(&cur_seq).collect();
            cur_seq = next_seq;
        }

        starts.into_iter().rev().reduce(|acc, x| x - acc)
    }
}

#[cfg(test)]
mod tests {
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
            .map(Sequence::from_str)
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.next())
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
            .map(Sequence::from_str)
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.prev())
            .sum::<i64>();
        assert_eq!(res, 2);
    }
}
