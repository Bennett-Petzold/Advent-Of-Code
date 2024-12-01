use std::{collections::HashMap, ops::Deref, str::FromStr};

use anyhow::{anyhow, bail};
use itertools::Itertools;
use num::{
    integer::{lcm, ExtendedGcd},
    Integer,
};
use ring_algorithm::is_coprime;

#[derive(Debug, PartialEq, Eq)]
pub enum Move {
    R,
    L,
}

impl TryFrom<char> for Move {
    type Error = anyhow::Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'R' => Ok(Self::R),
            'L' => Ok(Self::L),
            x => bail!("{x} is not 'R' or 'L'"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Directions(Vec<Move>);

impl FromStr for Directions {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let moves = s.chars().map(Move::try_from).try_collect()?;
        Ok(Self(moves))
    }
}

impl Deref for Directions {
    type Target = Vec<Move>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mappings {
    inner: HashMap<String, [String; 2]>,
}

impl Mappings {
    pub fn from_lines<I, S>(iter: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = iter.into_iter().map(|s| {
            let s = s.as_ref();
            let (tag, points) = s
                .split('=')
                .map(|part| part.trim())
                .collect_tuple()
                .ok_or(anyhow!("\"{s}\" has an incorrect number of '=' characters"))?;
            let points = points.replace(['(', ')', ' '], "");
            let (point_1, point_2) = points.split(',').collect_tuple().ok_or(anyhow!(
                "\"{points}\" has an incorrect number of ',' characters"
            ))?;
            Ok::<(String, [String; 2]), anyhow::Error>((
                tag.to_string(),
                [point_1.to_string(), point_2.to_string()],
            ))
        });

        let mut inner: HashMap<String, [String; 2]> = iter.try_collect()?;

        Ok(Self { inner })
    }

    /// Walks until ZZZ and returns number of steps
    pub fn walk(&self, dirs: &Directions) -> anyhow::Result<usize> {
        let mut dirs = dirs
            .iter()
            .map(|dir| match dir {
                Move::L => 0,
                Move::R => 1,
            })
            .cycle();

        let mut count = 0;
        let mut cur_pos = "AAA";

        while cur_pos != "ZZZ" {
            let map = self
                .inner
                .get(cur_pos)
                .ok_or(anyhow!("\"{cur_pos}\" does not have a mapping"))?;

            cur_pos = &map[dirs.next().unwrap()];
            count += 1;
        }

        Ok(count)
    }

    /// Walks from start until any position ending in Z
    pub fn walk_from<'a>(
        &'a self,
        start: &'a str,
        start_count: usize,
        dirs: &Directions,
    ) -> anyhow::Result<(usize, &'a str)> {
        let mut count = start_count;

        let mut next_pos = |pos: &str| -> anyhow::Result<&String> {
            let map = self
                .inner
                .get(pos)
                .ok_or(anyhow!("\"{pos}\" does not have a mapping"))?;

            let res = Ok(&map[match dirs[count % dirs.len()] {
                Move::L => 0,
                Move::R => 1,
            }]);
            count += 1;
            res
        };

        let mut cur_pos = next_pos(start)?;
        while !cur_pos.ends_with('Z') {
            cur_pos = next_pos(cur_pos)?;
        }

        Ok((count, &cur_pos))
    }

    /// Returns the first overlap as a gcf, if conditions are met
    ///
    /// There must be only one Z in all cycles, and it must always be the same
    fn gcf(z_idx: &[usize], cycles: &[Vec<usize>]) -> Option<usize> {
        if cycles
            .iter()
            .zip(z_idx)
            .all(|(cycle_set, z_idx)| cycle_set.len() == 1 && cycle_set[0] == *z_idx)
        {
            return Some(z_idx.iter().copied().reduce(lcm).unwrap());
        } else {
            None
        }
    }

    /// Chinese remainder theorem using the existence construction
    fn chinese_remainder_theorem<I, J, T>(remainders: I, modulos: J) -> Option<T>
    where
        I: IntoIterator<Item = T>,
        J: IntoIterator<Item = T>,
        T: Integer + Copy,
    {
        let mut pairs = remainders.into_iter().zip(modulos);
        let res = pairs.next();
        if let Some((mut value, mut m)) = res {
            pairs.for_each(|(pair_value, pair_m)| {
                let ExtendedGcd { gcd: _, x, y } = m.extended_gcd(&pair_m);
                (value, m) = (m * pair_m, value * y * pair_m + pair_value * x * m);
            });
            Some(value)
        } else {
            None
        }
    }

    /// Returns the lowest chinese remainder theorem result
    ///
    /// Requires at least one set of pairwise coprime cycles
    fn chinese_remainder_loop(z_idx: &[usize], cycles: &[Vec<usize>]) -> Option<usize> {
        let div_sets = cycles.iter().multi_cartesian_product();
        println!("div sets: {:?}", div_sets);
        div_sets
            .filter(|set| {
                set.iter()
                    .combinations(2)
                    .all(|comb| is_coprime(**comb[0], **comb[1]))
            })
            .filter_map(|set| {
                Self::chinese_remainder_theorem(z_idx.iter().copied(), set.into_iter().copied())
            })
            .min()
    }

    /// Returns the set of steps between each Z value in a cycle
    fn get_cycles(
        &self,
        z_vals: &[(usize, &str)],
        dirs: &Directions,
    ) -> anyhow::Result<Vec<Vec<usize>>> {
        z_vals
            .iter()
            .map(|(offset, pos)| {
                let mut cur_pos = *pos;
                let initial_offset = offset % dirs.len();
                let mut pattern_offset = initial_offset;

                (pattern_offset, cur_pos) = self.walk_from(cur_pos, pattern_offset, dirs)?;
                let mut cycle_set = vec![pattern_offset];

                while &cur_pos != pos || (pattern_offset % dirs.len()) != initial_offset {
                    (pattern_offset, cur_pos) = self.walk_from(cur_pos, pattern_offset, dirs)?;
                    cycle_set.push(pattern_offset);
                }

                for idx in cycle_set.len()..0 {
                    cycle_set[idx] -= cycle_set[idx - 1]
                }
                cycle_set[0] -= initial_offset;

                Ok::<_, anyhow::Error>(cycle_set)
            })
            .try_collect()
    }

    /// Walks until all nodes end with Z and returns number of steps
    pub fn ghost_walk(&self, dirs: &Directions) -> anyhow::Result<usize> {
        let mut z_vals: Vec<_> = self
            .inner
            .keys()
            .filter(|pos| pos.ends_with('A'))
            .map(|pos| self.walk_from(pos, 0, dirs))
            .try_collect()?;
        let z_idx = z_vals.iter().map(|(idx, _)| *idx).collect_vec();

        let cycles = self.get_cycles(&z_vals, dirs)?;

        if let Some(gcf_ret) = Self::gcf(&z_idx, &cycles) {
            println!("Result from greatest common factor");
            return Ok(gcf_ret);
        }

        if let Some(remainder_ret) = Self::chinese_remainder_loop(&z_idx, &cycles) {
            println!("Result from chinese remainder theorem");
            return Ok(remainder_ret);
        }

        eprintln!("BEWARE! Falling back to naive (computationally intensive) solution");

        let mut cycles = cycles
            .into_iter()
            .map(|c| c.into_iter().cycle())
            .collect_vec();

        // Keep increasing smaller counts until all counts are equal
        while z_vals
            .iter()
            .skip(1)
            .any(|(count, _)| count != &z_vals[0].0)
        {
            z_vals
                .iter_mut()
                .enumerate()
                .sorted_by_key(|(_, (val, _))| *val)
                .rev()
                .dedup()
                .skip(1)
                .for_each(|(idx, (val, _))| {
                    *val += cycles[idx].next().unwrap();
                });
            println!("New z vals: {:#?}", z_vals);
        }

        Ok(z_vals[0].0)
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
    fn parse_directions() {
        assert_eq!(
            Directions::from_str("RL").unwrap(),
            Directions(vec![Move::R, Move::L])
        );
    }

    #[test]
    fn parse_mappings() {
        let input = ["AAA = (BBB, BBB)", "BBB = (AAA, ZZZ)", "ZZZ = (ZZZ, ZZZ)"];
        assert_eq!(
            Mappings::from_lines(input).unwrap(),
            Mappings {
                inner: [
                    ("AAA".to_string(), ["BBB".to_string(), "BBB".to_string()]),
                    ("BBB".to_string(), ["AAA".to_string(), "ZZZ".to_string()]),
                    ("ZZZ".to_string(), ["ZZZ".to_string(), "ZZZ".to_string()])
                ]
                .into_iter()
                .collect()
            }
        );
    }

    #[test]
    fn part1_test1() {
        let mut input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());

        let dirs = Directions::from_str(&input.next().unwrap()).unwrap();
        let maps = Mappings::from_lines(input.skip(1)).unwrap();
        assert_eq!(maps.walk(&dirs).unwrap(), 2);
    }

    #[test]
    fn part1_test2() {
        let mut input = BufReader::new(File::open("test-input-2").unwrap())
            .lines()
            .map(|line| line.unwrap());

        let dirs = Directions::from_str(&input.next().unwrap()).unwrap();
        let maps = Mappings::from_lines(input.skip(1)).unwrap();
        assert_eq!(maps.walk(&dirs).unwrap(), 6);
    }

    #[test]
    fn part2() {
        let mut input = BufReader::new(File::open("test-input-3").unwrap())
            .lines()
            .map(|line| line.unwrap());

        let dirs = Directions::from_str(&input.next().unwrap()).unwrap();
        let maps = Mappings::from_lines(input.skip(1)).unwrap();
        assert_eq!(maps.ghost_walk(&dirs).unwrap(), 6);
    }
}
