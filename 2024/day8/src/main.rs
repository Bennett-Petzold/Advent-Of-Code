use std::collections::{HashMap, HashSet};

use advent_rust_lib::{grid::Pos2D, read::input, signed::SignedUsize};

fn main() {
    let map = AntennaMap::from_str_iter(input());
    part_1(&map);
    part_2(&map);
}

fn part_1(map: &AntennaMap) {
    let impacted_zones: HashSet<_> = map.antenna_impacted_zones().collect();
    println!("{}", impacted_zones.len());
}

fn part_2(map: &AntennaMap) {
    let impacted_zones: HashSet<_> = map.antenna_infinite_impacted_zones().collect();
    println!("{}", impacted_zones.len());
}

#[derive(Debug)]
pub struct AntennaMap {
    antenna: HashMap<char, Vec<Pos2D>>,
    x_limit: usize,
    y_limit: usize,
}

impl AntennaMap {
    pub fn from_str_iter<S, I>(iter: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let mut y_val = 0;
        let mut x_limit = 0;
        let mut antenna = HashMap::new();

        let mut iter = iter.into_iter();
        if let Some(first_line) = iter.by_ref().next() {
            x_limit = first_line.as_ref().len();

            for line in std::iter::once(first_line).chain(iter) {
                let antenna_iter = line
                    .as_ref()
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| *c != '.')
                    .map(|(x, c)| (c, Pos2D::new(x, y_val)));

                for (c, pos) in antenna_iter {
                    antenna.entry(c).or_insert(Vec::new()).push(pos);
                }
                y_val += 1;
            }
        }

        Self {
            antenna,
            x_limit,
            y_limit: y_val,
        }
    }

    pub fn antenna_impacted_zones(&self) -> impl Iterator<Item = Pos2D> + use<'_> {
        self.antenna.iter().flat_map(move |(_, antenna_locs)| {
            (0..(antenna_locs.len().saturating_sub(1))).flat_map(move |idx| {
                ((idx + 1)..antenna_locs.len()).flat_map(move |other_idx| {
                    self.generated_pos(antenna_locs[idx], antenna_locs[other_idx])
                })
            })
        })
    }

    pub fn antenna_infinite_impacted_zones(&self) -> impl Iterator<Item = Pos2D> + use<'_> {
        self.antenna.iter().flat_map(move |(_, antenna_locs)| {
            (0..(antenna_locs.len().saturating_sub(1))).flat_map(move |idx| {
                ((idx + 1)..antenna_locs.len()).flat_map(move |other_idx| {
                    self.infinite_generated_pos(antenna_locs[idx], antenna_locs[other_idx])
                })
            })
        })
    }

    fn valid_pos(&self, pos: Pos2D) -> bool {
        (pos.x < self.x_limit) && (pos.y < self.y_limit)
    }

    fn generated_pos(&self, lhs: Pos2D, rhs: Pos2D) -> impl Iterator<Item = Pos2D> + use<'_> {
        lhs.flip(rhs)
            .into_iter()
            .chain(rhs.flip(lhs))
            .filter(|pos| self.valid_pos(*pos))
    }

    fn infinite_generated_pos(
        &self,
        lhs: Pos2D,
        rhs: Pos2D,
    ) -> impl Iterator<Item = Pos2D> + use<'_> {
        let gen_while_valid =
            |origin: Pos2D, pivot| InfiniteGenPosIter::new(self, origin.flip(pivot), pivot);

        [lhs, rhs]
            .into_iter()
            .chain(gen_while_valid(lhs, rhs).chain(gen_while_valid(rhs, lhs)))
    }
}

#[derive(Debug)]
struct InfiniteGenPosIter<'a> {
    map: &'a AntennaMap,
    cur_val: Option<Pos2D>,
    x_step: SignedUsize,
    y_step: SignedUsize,
}

impl<'a> InfiniteGenPosIter<'a> {
    pub fn new(map: &'a AntennaMap, cur_val: Option<Pos2D>, prev_val: Pos2D) -> Self {
        Self {
            map,
            cur_val,
            x_step: cur_val
                .and_then(|val| SignedUsize::from(val.x) - SignedUsize::from(prev_val.x))
                .unwrap_or(SignedUsize::from(0)),
            y_step: cur_val
                .and_then(|val| SignedUsize::from(val.y) - SignedUsize::from(prev_val.y))
                .unwrap_or(SignedUsize::from(0)),
        }
    }
}

impl Iterator for InfiniteGenPosIter<'_> {
    type Item = Pos2D;

    fn next(&mut self) -> Option<Self::Item> {
        let resolved_val = self.cur_val?;
        if self.map.valid_pos(resolved_val) {
            self.cur_val = (resolved_val.x + self.x_step)
                .and_then(|x| Some(Pos2D::new(x, (resolved_val.y + self.y_step)?)));
            Some(resolved_val)
        } else {
            None
        }
    }
}
