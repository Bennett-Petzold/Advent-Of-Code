use std::{
    cmp::{min, Reverse},
    collections::{BinaryHeap, HashMap, HashSet},
    env::args,
};

use advent_rust_lib::{
    direction::Direction,
    grid::{Pos2D, RectangleGrid},
    ll::{PointerSequence, PointerSequenceInternal, RevLinkedNode, RevLinkedNodeInternal},
    read::filtered_input,
};

fn main() {
    let target_save: u64 = str::parse(&args().nth(1).unwrap()).unwrap();
    let track = Track::from_input_iter(filtered_input(&[2])).unwrap();

    part_1(&track, target_save);
}

fn part_1(track: &Track, target_save: u64) {
    println!("{}", track.num_valid_cheats(target_save));
}

// -------------------------------------------------- //

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CostNode {
    cur_pos: Pos2D,
    wall: Option<PointerSequence<u64>>,
}

#[derive(Debug)]
struct Track {
    grid: RectangleGrid<bool>,
    start: Pos2D,
    end: Pos2D,
}

impl Track {
    pub fn from_input_iter<S, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut start = None;
        let mut end = None;

        let iter = iter
            .into_iter()
            .enumerate()
            .inspect(|(y, line)| {
                if let Some(x) = line.as_ref().chars().position(|c| c == 'S') {
                    start = Some(Pos2D::new(x, *y))
                }
            })
            .inspect(|(y, line)| {
                if let Some(x) = line.as_ref().chars().position(|c| c == 'E') {
                    end = Some(Pos2D::new(x, *y))
                }
            })
            .map(|(_, line)| line.as_ref().chars().map(|c| c == '#').collect::<Vec<_>>());

        let grid = RectangleGrid::try_from_iter(iter).ok()?;
        let start = start?;
        let end = end?;

        Some(Self { grid, start, end })
    }

    pub fn num_valid_cheats(&self, target_save: u64) -> u64 {
        let mut visited = HashSet::new();
        let mut cheating_visited = HashSet::new();
        let mut next_cheating_visited = Vec::new();
        let mut to_visit = vec![CostNode {
            cur_pos: self.start,
            wall: None,
        }];

        let mut passing = Vec::new();

        for idx in 0..u64::MAX {
            println!("{idx}");
            for next_node in std::mem::take(&mut to_visit) {
                if next_node.cur_pos == self.end {
                    if let Some(wall) = next_node.wall {
                        for x in 0..*PointerSequenceInternal::resolve(&wall) {
                            passing.push(idx);
                        }
                    } else {
                        let max_cost = idx - target_save;
                        println!("Passing: {:#?}", passing);
                        return passing
                            .into_iter()
                            .take_while(|pass| *pass < max_cost)
                            .count() as u64;
                    }
                } else {
                    if let Some(wall) = next_node.wall {
                        next_cheating_visited.push((next_node.cur_pos, next_node.wall));
                    } else {
                        visited.insert(next_node.cur_pos);
                    }

                    let new_visits = Direction::all()
                        .into_iter()
                        .flat_map(|dir| next_node.cur_pos.step_dir(dir))
                        .flat_map(|new_pos| {
                            if let Some(is_wall) = self.grid.get(new_pos).copied() {
                                if !(next_node.wall.is_some() && is_wall) {
                                    let wall = if let Some(existing_wall) = next_node.wall {
                                        Some(PointerSequenceInternal::point(existing_wall))
                                    } else if is_wall {
                                        Some(PointerSequenceInternal::new(1))
                                    } else {
                                        None
                                    };
                                    let new_node = CostNode {
                                        cur_pos: new_pos,
                                        wall,
                                    };
                                    if !visited.contains(&new_node.cur_pos)
                                        && (!wall || !cheating_visited.contains(&new_node.cur_pos))
                                    {
                                        Some(new_node)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        });
                    to_visit.extend(new_visits);
                }
            }

            cheating_visited.extend(next_cheating_visited.drain(..));
        }

        0
    }
}
