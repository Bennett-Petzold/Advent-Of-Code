use std::collections::HashSet;

use derive_getters::Getters;
use itertools::Itertools;

#[derive(Debug, Clone, Getters)]
pub struct NumCandidate {
    value: u32,
    pos: ([usize; 2], usize),
}

impl NumCandidate {
    pub fn neighbors(&self) -> Vec<[usize; 2]> {
        self.pos()
            .0
            .iter()
            .flat_map(|x| {
                vec![
                    [x.saturating_sub(1), self.pos().1.saturating_sub(1)],
                    [x.saturating_sub(1), self.pos().1],
                    [x.saturating_sub(1), self.pos().1 + 1],
                    [*x, self.pos().1.saturating_sub(1)],
                    [*x, self.pos().1 + 1],
                    [x + 1, self.pos().1.saturating_sub(1)],
                    [x + 1, self.pos().1],
                    [x + 1, self.pos().1 + 1],
                ]
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct EngineEntries {
    candidates: Vec<NumCandidate>,
    symbols: HashSet<[usize; 2]>,
    gears: HashSet<[usize; 2]>,
}

impl<S: AsRef<str>> FromIterator<S> for EngineEntries {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let size_hint = iter.size_hint().1.unwrap_or(iter.size_hint().0);
        let size_hint = size_hint * size_hint;
        let mut candidates = Vec::with_capacity(size_hint);
        let mut symbols = HashSet::with_capacity(size_hint);
        let mut gears = HashSet::with_capacity(size_hint);

        iter.enumerate().for_each(|(y, s)| {
            let s = s.as_ref();
            let mut head = 0;

            while head < s.len() {
                let digit_len = s[head..].bytes().take_while(|x| x.is_ascii_digit()).count();
                if digit_len > 0 {
                    candidates.push(NumCandidate {
                        value: s[head..head + digit_len].parse().unwrap(),
                        pos: ([head, head + digit_len - 1], y),
                    });
                    head += digit_len;
                } else {
                    if &s[head..head + 1] != "." {
                        if &s[head..head + 1] == "*" {
                            gears.insert([head, y]);
                        } else {
                            symbols.insert([head, y]);
                        }
                    }
                    head += 1;
                }
            }
        });

        Self {
            candidates,
            symbols,
            gears,
        }
    }
}

impl EngineEntries {
    pub fn part_numbers(&self) -> impl Iterator<Item = u32> + '_ {
        self.candidates
            .iter()
            .filter(|cand| {
                cand.neighbors()
                    .into_iter()
                    .any(|x| self.symbols.contains(&x) || self.gears.contains(&x))
            })
            .map(|cand| *cand.value())
    }

    pub fn gears(&self) -> Vec<u32> {
        self.candidates
            .iter()
            .flat_map(|cand| {
                cand.neighbors()
                    .into_iter()
                    .filter_map(|x| self.gears.get(&x))
                    .unique()
                    .map(|gear| (gear, cand.clone()))
            })
            .sorted_by(|(lhs, _), (rhs, _)| lhs.partial_cmp(rhs).unwrap())
            .group_by(|(gear, _)| *gear)
            .into_iter()
            .map(|(_, group)| group)
            .filter_map(|group| {
                let mut count = 0;
                let ret = group
                    .into_iter()
                    .map(|(_, neighbor)| {
                        count += 1;
                        *neighbor.value()
                    })
                    .product();
                if count == 2 {
                    Some(ret)
                } else {
                    None
                }
            })
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_part1() {
        let input = vec![
            "467..114..",
            "...*......",
            "..35..633.",
            "......#...",
            "617*......",
            ".....+.58.",
            "..592.....",
            "......755.",
            "...$.*....",
            ".664.598..",
        ];

        let entries = EngineEntries::from_iter(input);
        println!("Entries: {:#?}", entries);

        assert_eq!(entries.part_numbers().sum::<u32>(), 4361);
    }

    #[test]
    fn sum_part2() {
        let input = vec![
            "467..114..",
            "...*......",
            "..35..633.",
            "......#...",
            "617*......",
            ".....+.58.",
            "..592.....",
            "......755.",
            "...$.*....",
            ".664.598..",
        ];

        let entries = EngineEntries::from_iter(input);
        println!("Entries: {:#?}", entries);

        assert_eq!(entries.gears().into_iter().sum::<u32>(), 467835);
    }
}
