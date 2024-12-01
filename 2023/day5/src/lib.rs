use std::{
    cmp::{max, min},
    ops::{Deref, Range},
};

use anyhow::anyhow;
use itertools::Itertools;

#[derive(Debug, PartialEq, Eq)]
pub struct ConvSet {
    dest: Range<u64>,
    source: Range<u64>,
}

// Returns the range covered by both lhs and rhs
fn range_overlap(lhs: &Range<u64>, rhs: &Range<u64>) -> Option<Range<u64>> {
    let start = max(lhs.start, rhs.start);
    let end = min(lhs.end, rhs.end);

    if start < end {
        Some(start..end)
    } else {
        None
    }
}

impl ConvSet {
    pub fn convert(&self, val: u64) -> Option<u64> {
        if self.source.contains(&val) {
            Some(self.dest.end - (self.source.end - val))
        } else {
            None
        }
    }

    pub fn convert_range(&self, val: &Range<u64>) -> Option<Range<u64>> {
        let overlap = range_overlap(&self.source, val)?;

        let offset_start = overlap.start - self.source.start;
        let offset_end = self.source.end - overlap.end;

        Some((self.dest.start + offset_start)..(self.dest.end - offset_end))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConvMap {
    sets: Vec<ConvSet>,
}

impl ConvMap {
    pub fn convert(&self, val: u64) -> u64 {
        self.sets.iter().find_map(|s| s.convert(val)).unwrap_or(val)
    }

    pub fn convert_range(&self, val: &[Range<u64>]) -> Vec<Range<u64>> {
        val.iter()
            .cloned()
            .flat_map(|input_range| {
                let outputs = self
                    .sets
                    .iter()
                    .filter_map(|set| set.convert_range(&input_range))
                    .collect_vec();
                outputs
            })
            .collect()
    }
}

impl Deref for ConvMap {
    type Target = Vec<ConvSet>;
    fn deref(&self) -> &Self::Target {
        &self.sets
    }
}

fn split_range(to_split: &Range<u64>, splitter: &Range<u64>) -> Vec<Range<u64>> {
    if let Some(overlap) = range_overlap(to_split, splitter) {
        vec![to_split.start..overlap.start, overlap.end..to_split.end]
    } else {
        vec![to_split.clone()]
    }
}

fn split_ranges(ranges: &[Range<u64>], splitter: &Range<u64>) -> Vec<Range<u64>> {
    ranges
        .iter()
        .flat_map(|to_split| split_range(to_split, splitter))
        .collect()
}

impl<S: AsRef<str>> FromIterator<S> for ConvMap {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut sets = iter
            .into_iter()
            .map(|x| {
                let (dest_start, src_start, len) = x
                    .as_ref()
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse())
                    .collect_tuple()
                    .ok_or(anyhow!(""))?;
                let (dest_start, src_start, len) = (dest_start?, src_start?, len?);
                Ok(ConvSet {
                    dest: dest_start..dest_start + len,
                    source: src_start..src_start + len,
                })
            })
            .take_while(|set_status| set_status.is_ok())
            .map(|set: anyhow::Result<ConvSet>| set.unwrap())
            .collect_vec();

        // Cover all other values, keeping track of discontinuities
        let mut ranges = vec![u64::min_value()..u64::max_value()];

        sets.iter()
            .for_each(|s| ranges = split_ranges(&ranges, &s.source));
        sets.append(
            &mut ranges
                .into_iter()
                .map(|x| ConvSet {
                    dest: x.clone(),
                    source: x.clone(),
                })
                .collect(),
        );

        Self { sets }
    }
}

impl ConvMap {
    pub fn convs_from_almanac<I, S: AsRef<str>>(iter: I) -> Vec<Self>
    where
        I: IntoIterator<Item = S>,
    {
        let mut iter = iter
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .filter(|s| !s.is_empty())
            .skip(1)
            .peekable();

        let mut collected = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
        while iter.peek().is_some() {
            collected.push(ConvMap::from_iter(iter.by_ref()))
        }
        collected
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Almanac {
    seeds: Vec<u64>,
    convs: Vec<ConvMap>,
}

impl Almanac {
    pub fn from_almanac_iter<I, S: AsRef<str>>(iter: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
    {
        let mut iter = iter.into_iter();
        let seeds = iter
            .next()
            .ok_or(anyhow!("0 line input"))?
            .as_ref()
            .split(':')
            .nth(1)
            .ok_or(anyhow!("No colon on seeds line"))?
            .split(' ')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        let convs = ConvMap::convs_from_almanac(iter);

        Ok(Self { seeds, convs })
    }

    pub fn locations(&self) -> impl Iterator<Item = u64> + '_ {
        self.seeds.iter().map(|seed| {
            let mut cur_val = *seed;
            self.convs
                .iter()
                .for_each(|next_conv| cur_val = next_conv.convert(cur_val));
            cur_val
        })
    }

    pub fn range_locations(&self) -> impl Iterator<Item = Option<u64>> + '_ {
        Self::pairs(&self.seeds).map(|seed_range| {
            let mut cur_val = vec![seed_range];
            self.convs.iter().for_each(|next_conv| {
                cur_val = next_conv.convert_range(&cur_val);
            });
            cur_val.into_iter().map(|r| r.start).min()
        })
    }

    fn pairs(item: &[u64]) -> impl Iterator<Item = Range<u64>> + '_ {
        item.iter()
            .step_by(2)
            .zip(item.iter().skip(1).step_by(2))
            .map(|(start, len)| (*start..start + len))
    }

    pub fn seeds_as_pairs(&mut self) {
        self.seeds = Self::pairs(&self.seeds).flatten().collect();
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
    fn read_lines() {
        let input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());
        let almanac = Almanac::from_almanac_iter(input).unwrap();

        assert_eq!(almanac.seeds, vec![79, 14, 55, 13]);

        assert_eq!(
            almanac.convs[0][0],
            ConvSet {
                dest: 50..52,
                source: 98..100
            }
        );
    }

    #[test]
    fn part1_test() {
        let input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());
        let almanac = Almanac::from_almanac_iter(input).unwrap();
        assert_eq!(almanac.locations().min().unwrap(), 35);
    }

    #[test]
    fn pairs_test() {
        let input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());
        let mut almanac = Almanac::from_almanac_iter(input).unwrap();
        almanac.seeds_as_pairs();

        assert_eq!(almanac.seeds[1], 80);
    }

    #[test]
    fn part2_test_naive() {
        let input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());
        let mut almanac = Almanac::from_almanac_iter(input).unwrap();
        almanac.seeds_as_pairs();
        assert_eq!(almanac.locations().min().unwrap(), 46);
    }

    #[test]
    fn part2_test() {
        let input = BufReader::new(File::open("test-input").unwrap())
            .lines()
            .map(|line| line.unwrap());
        let almanac = Almanac::from_almanac_iter(input).unwrap();
        assert_eq!(almanac.range_locations().flatten().min().unwrap(), 46);
    }
}
