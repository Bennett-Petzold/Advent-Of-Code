use std::str::FromStr;

use anyhow::{anyhow, bail};
use derive_getters::Getters;
use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Card {
    Number(u8),
    T,
    J,
    Q,
    K,
    A,
}

impl TryFrom<char> for Card {
    type Error = anyhow::Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        if let Some(digit) = value.to_digit(10) {
            Ok(Self::Number(digit as u8))
        } else {
            match value {
                'T' => Ok(Self::T),
                'J' => Ok(Self::J),
                'Q' => Ok(Self::Q),
                'K' => Ok(Self::K),
                'A' => Ok(Self::A),
                _ => bail!("'{value}' is not a valid card face"),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl From<&mut [Card; 5]> for HandType {
    fn from(value: &mut [Card; 5]) -> Self {
        let pre_groups = value.iter().sorted().group_by(|x| *x);
        let mut groups: Vec<_> = pre_groups.into_iter().map(|(_, group)| group).collect();

        match groups.len() {
            1 => Self::FiveOfAKind,
            2 => {
                if groups[0].by_ref().count() == 4 || groups[1].by_ref().count() == 4 {
                    Self::FourOfAKind
                } else {
                    Self::FullHouse
                }
            }
            3 => {
                if groups.into_iter().any(|mut g| g.by_ref().count() == 3) {
                    Self::ThreeOfAKind
                } else {
                    Self::TwoPair
                }
            }
            4 => Self::OnePair,
            5 => Self::HighCard,
            x => panic!("Impossible number of groups: {x}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WildCardHandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl From<&mut [Card; 5]> for WildCardHandType {
    fn from(value: &mut [Card; 5]) -> Self {
        let value_clone = value.clone();
        let pre_groups = value_clone.iter().sorted().group_by(|x| *x);
        let mut groups: Vec<_> = pre_groups
            .into_iter()
            .map(|(key, mut group)| (key, group.by_ref().count()))
            .collect();

        if let Some(jokers) = groups
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, (key, _))| key == &&Card::J)
            .map(|(idx, (_, count))| (idx, count))
            .next()
        {
            groups.remove(jokers.0);
            if let Some(max_entry) = groups
                .iter()
                .enumerate()
                .max_by(|(_, (_, lhs)), (_, (_, rhs))| lhs.cmp(rhs))
                .map(|(idx, _)| idx)
            {
                groups[max_entry].1 += jokers.1;
            } else {
                groups.push((&Card::J, 5));
            }

            value
                .iter_mut()
                .filter(|card| card == &&Card::J)
                .for_each(|card| *card = Card::Number(1));
        }

        match groups.len() {
            1 => Self::FiveOfAKind,
            2 => {
                if groups[0].1 == 4 || groups[1].1 == 4 {
                    Self::FourOfAKind
                } else {
                    Self::FullHouse
                }
            }
            3 => {
                if groups.into_iter().any(|g| g.1 == 3) {
                    Self::ThreeOfAKind
                } else {
                    Self::TwoPair
                }
            }
            4 => Self::OnePair,
            5 => Self::HighCard,
            x => panic!("Impossible number of groups: {x}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct Hand<R> {
    rank: R,
    cards: [Card; 5],
    bid: usize,
}

impl<R: for<'a> From<&'a mut [Card; 5]>> FromStr for Hand<R> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cards, bid) = s
            .split_whitespace()
            .collect_tuple()
            .ok_or(anyhow!("{s} is incorrectly formatted"))?;

        let mut cards = cards
            .chars()
            .map(Card::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?
            .try_into()
            .map_err(|x| anyhow!("{:?}", x))?;

        let rank = R::from(&mut cards);
        let bid = bid.parse()?;

        Ok(Self { rank, cards, bid })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    #[test]
    fn card_ordering() {
        assert!(Card::A > Card::K);

        let mut cards = vec![Card::Number(8), Card::K, Card::Number(3), Card::A];
        cards.sort_unstable();
        assert_eq!(
            cards,
            vec![Card::Number(3), Card::Number(8), Card::K, Card::A]
        );
    }

    #[test]
    fn hand_parsing() {
        assert_eq!(
            Hand::<HandType>::from_str("11111 000").unwrap().rank(),
            &HandType::FiveOfAKind
        );
        assert_eq!(
            Hand::<HandType>::from_str("11112 000").unwrap().rank(),
            &HandType::FourOfAKind
        );
        assert_eq!(
            Hand::<HandType>::from_str("11123 000").unwrap().rank(),
            &HandType::ThreeOfAKind
        );
        assert_eq!(
            Hand::<HandType>::from_str("11122 000").unwrap().rank(),
            &HandType::FullHouse
        );
        assert_eq!(
            Hand::<HandType>::from_str("11223 000").unwrap().rank(),
            &HandType::TwoPair
        );
        assert_eq!(
            Hand::<HandType>::from_str("11234 000").unwrap().rank(),
            &HandType::OnePair
        );
        assert_eq!(
            Hand::<HandType>::from_str("12345 000").unwrap().rank(),
            &HandType::HighCard
        );
    }

    #[test]
    fn hand_ordering() {
        let hand_smaller = Hand::<HandType>::from_str("87776 32423").unwrap();
        let hand_larger = Hand::<HandType>::from_str("87779 32423").unwrap();
        assert!(hand_smaller < hand_larger);

        assert!(
            Hand::<HandType>::from_str("33332 324").unwrap() > Hand::from_str("2AAAA 324").unwrap()
        );
        assert!(
            Hand::<HandType>::from_str("11223 324").unwrap() > Hand::from_str("12345 324").unwrap()
        );
        assert!(
            Hand::<HandType>::from_str("11223 324").unwrap() > Hand::from_str("12345 324").unwrap()
        );
    }

    #[test]
    fn part1() {
        let answer: usize = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| Hand::<HandType>::from_str(&line.unwrap()).unwrap())
            .sorted()
            .enumerate()
            .map(|(idx, hand)| (idx + 1) * hand.bid())
            .sum();
        assert_eq!(answer, 6440);
    }

    #[test]
    /// https://www.reddit.com/r/adventofcode/comments/18cr4xr/2023_day_7_better_example_input_not_a_spoiler/
    fn extended_testing() {
        let input = [
            "2345A 1", "Q2KJJ 13", "Q2Q2Q 19", "T3T3J 17", "T3Q33 11", "2345J 3", "J345A 2",
            "32T3K 5", "T55J5 29", "KK677 7", "KTJJT 34", "QQQJA 31", "JJJJJ 37", "JAAAA 43",
            "AAAAJ 59", "AAAAA 61", "2AAAA 23", "2JJJJ 53", "JJJJ2 41",
        ];
        let mut input: Vec<_> = input
            .into_iter()
            .map(Hand::<HandType>::from_str)
            .try_collect()
            .unwrap();
        input.sort_unstable();

        let output = [
            "2345J 3", "2345A 1", "J345A 2", "32T3K 5", "Q2KJJ 13", "T3T3J 17", "KTJJT 34",
            "KK677 7", "T3Q33 11", "T55J5 29", "QQQJA 31", "Q2Q2Q 19", "2JJJJ 53", "2AAAA 23",
            "JJJJ2 41", "JAAAA 43", "AAAAJ 59", "JJJJJ 37", "AAAAA 61",
        ];
        let output: Vec<_> = output
            .into_iter()
            .map(Hand::<HandType>::from_str)
            .try_collect()
            .unwrap();

        assert_eq!(input, output);
        assert_eq!(
            input
                .iter()
                .enumerate()
                .map(|(idx, hand)| (idx + 1) * hand.bid())
                .sum::<usize>(),
            6592
        );
    }

    #[test]
    fn part2() {
        let answer: usize = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| Hand::<WildCardHandType>::from_str(&line.unwrap()).unwrap())
            .sorted()
            .enumerate()
            .map(|(idx, hand)| (idx + 1) * hand.bid())
            .sum();
        assert_eq!(answer, 5905);
    }
}
