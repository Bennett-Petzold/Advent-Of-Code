use std::collections::HashSet;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::bail;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Card {
    winning: HashSet<u32>,
    held: Vec<u32>,
}

fn nums_from_string(s: &str) -> impl Iterator<Item = Result<u32, ParseIntError>> + '_ {
    s.split(' ')
        .filter(|x| !x.is_empty())
        .map(|x| x.parse::<u32>())
}

impl AsRef<Self> for Card {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl FromStr for Card {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if &s[0..5] != "Card " {
            bail!("\"{s}\" does not start with \"Card \"")
        };
        let s = &s[5..];

        let s = &s[s.find(':').ok_or(anyhow!("\"{s}\" does not have \":\""))? + 1..];
        let (winning, held) = s
            .split('|')
            .collect_tuple()
            .ok_or(anyhow!("\"{s}\" has the wrong number of \"|\" characters"))?;

        Ok(Self {
            winning: nums_from_string(winning).try_collect()?,
            held: nums_from_string(held).try_collect()?,
        })
    }
}

impl Card {
    pub fn match_count(&self) -> usize {
        self.held
            .iter()
            .filter(|x| self.winning.contains(x))
            .count()
    }

    pub fn match_points(&self) -> usize {
        let count = self.match_count();
        if count > 0 {
            1 << (count - 1) // 2 ^ ( count - 1 )
        } else {
            0
        }
    }

    pub fn total_scratchcards<I, T: AsRef<Self>>(iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut count_data: Vec<_> = iter
            .into_iter()
            .map(|card| (1, card.as_ref().match_count()))
            .collect();

        for idx in 0..count_data.len() {
            let (count, matches) = count_data[idx];
            for entry in 1..=matches {
                count_data[idx + entry].0 += count;
            }
        }

        count_data.into_iter().map(|(count, _)| count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_sum() {
        let input = [
            "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53",
            "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19",
            "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1",
            "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83",
            "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36",
            "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11",
        ];

        let res: usize = input
            .into_iter()
            .map(Card::from_str)
            .collect::<anyhow::Result<Vec<Card>>>()
            .unwrap()
            .into_iter()
            .map(|card| card.match_points())
            .sum();
        assert_eq!(res, 13)
    }

    #[test]
    fn part2_sum() {
        let input = [
            "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53",
            "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19",
            "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1",
            "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83",
            "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36",
            "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11",
        ];

        let res = Card::total_scratchcards(
            input
                .into_iter()
                .map(Card::from_str)
                .collect::<anyhow::Result<Vec<Card>>>()
                .unwrap(),
        );
        assert_eq!(res, 30)
    }
}
