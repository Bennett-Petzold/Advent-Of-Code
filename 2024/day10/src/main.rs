use std::collections::HashSet;

use advent_rust_lib::{
    grid::{Direction, Pos2D},
    read::input,
};

fn main() {
    let trail = Trail::from_str_iter(input()).unwrap();
    part_1(trail.clone());
    part_2(trail);
}

fn part_1(trail: Trail) {
    println!("{}", trail.num_unique_paths());
}

fn part_2(trail: Trail) {
    println!("{}", trail.num_paths());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrailLoc {
    pub height: u8,
}

impl TrailLoc {
    pub fn new(height: Option<u8>) -> Option<Self> {
        if let Some(height) = height {
            (height < 10).then_some(Self { height })
        } else {
            Some(Self { height: 10 })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trail {
    // 2D array
    arr: Box<[TrailLoc]>,
    x_limit: usize,
    y_limit: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct NonDigitErr;

impl Trail {
    pub fn from_str_iter<S, I>(iter: I) -> Result<Self, NonDigitErr>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let mut iter = iter.into_iter();

        let mut x_limit = 0;
        let mut arr = Vec::new();

        if let Some(first) = iter.next() {
            x_limit = first.as_ref().len();

            arr = std::iter::once(first)
                .chain(iter)
                .flat_map(|line| {
                    line.as_ref()
                        .chars()
                        .map(|x| TrailLoc::new(char::to_digit(x, 10).map(|y| y as u8)))
                        .collect::<Vec<_>>()
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(NonDigitErr)?;
        };

        let y_limit = arr.len() / x_limit;
        Ok(Self {
            arr: arr.into_boxed_slice(),
            x_limit,
            y_limit,
        })
    }

    fn expand_point(&self, point: usize) -> Pos2D {
        let x = point % self.x_limit;
        let y = point / self.x_limit;

        Pos2D { x, y }
    }

    fn flatten_point(&self, point: Pos2D) -> usize {
        (point.y * self.x_limit) + point.x
    }

    fn point_within(&self, point: Pos2D) -> bool {
        (point.x < self.x_limit) && (point.y < self.y_limit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ValuedLoc {
    pos: Pos2D,
    count: u64,
}

impl ValuedLoc {
    fn from_nine(trail: &Trail, pos: usize) -> Self {
        let pos = trail.expand_point(pos);
        Self { pos, count: 1 }
    }
}

impl Trail {
    fn surrounding_valid(&self, point: Pos2D, height: u8) -> impl Iterator<Item = Pos2D> + use<'_> {
        Direction::all()
            .into_iter()
            .flat_map(move |dir| point.step_dir(dir))
            .filter(|next_point| self.point_within(*next_point))
            .filter(move |next_point| self.arr[self.flatten_point(*next_point)].height == height)
    }

    pub fn num_paths(&self) -> u64 {
        let mut endpoints: Vec<_> = self
            .arr
            .iter()
            .enumerate()
            .filter(|(_, loc)| (loc.height == 9))
            .map(|(idx, _)| ValuedLoc::from_nine(self, idx))
            .collect();

        for height in (0..9).rev() {
            // Collect endpoints into a new vector
            let mut new_endpoints: Vec<ValuedLoc> = Vec::with_capacity(endpoints.len() * 4);

            // Generate all valid endpoints, draining the existing vector directly
            let new_endpoint_iter = std::mem::take(&mut endpoints).into_iter().flat_map(|loc| {
                self.surrounding_valid(loc.pos, height)
                    .map(move |next_point| ValuedLoc {
                        pos: next_point,
                        count: loc.count,
                    })
            });

            // Combine converging endpoints
            for endpoint in new_endpoint_iter {
                match new_endpoints.binary_search_by_key(&endpoint.pos, |x| x.pos) {
                    Ok(idx) => new_endpoints[idx].count += endpoint.count,
                    Err(idx) => new_endpoints.insert(idx, endpoint),
                }
            }

            // Assign with the newly generated values
            endpoints = new_endpoints;
        }

        // Sum all values at zero
        endpoints.into_iter().map(|loc| loc.count).sum()
    }

    pub fn num_unique_paths(&self) -> usize {
        let mut endpoints: Vec<_> = self
            .arr
            .iter()
            .enumerate()
            .filter(|(_, loc)| (loc.height == 9))
            .map(|(idx, _)| self.expand_point(idx))
            .enumerate()
            .map(|(unique_id, loc)| (loc, HashSet::from([unique_id])))
            .collect();

        for height in (0..9).rev() {
            // Collect endpoints into a new vector
            let mut new_endpoints: Vec<(Pos2D, HashSet<usize>)> =
                Vec::with_capacity(endpoints.len() * 4);

            // Generate all valid endpoints, draining the existing vector directly
            let new_endpoint_iter =
                std::mem::take(&mut endpoints)
                    .into_iter()
                    .flat_map(|(loc, ids)| {
                        self.surrounding_valid(loc, height)
                            .map(move |next_point| (next_point, ids.clone()))
                    });

            // Combine converging endpoints
            for endpoint in new_endpoint_iter {
                match new_endpoints.binary_search_by_key(&endpoint.0, |x| x.0) {
                    Ok(idx) => new_endpoints[idx].1.extend(endpoint.1),
                    Err(idx) => new_endpoints.insert(idx, endpoint),
                }
            }

            // Assign with the newly generated values
            endpoints = new_endpoints;
        }

        // Sum all values at zero
        endpoints.into_iter().map(|(_, set)| set.len()).sum()
    }
}
