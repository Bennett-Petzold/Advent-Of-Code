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
enum EndPos {
    CountOff(usize),
    Resolved(RevLinkedNode),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Walls {
    start: Pos2D,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CostNode {
    cur_pos: Pos2D,
    // start, end
    walls: EndPos,
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
        let mut visited = HashMap::new();
        let mut to_visit = vec![CostNode {
            cur_pos: self.start,
            walls: EndPos::Resolved(None),
        }];

        let mut passing = HashMap::new();

        for idx in 0..u64::MAX {
            println!("{idx}");
            for next_node in std::mem::take(&mut to_visit) {
                if next_node.cur_pos == self.end {
                    if let EndPos::Resolved(Some(node)) = next_node.walls {
                        for (wall_start, wall_end) in next_node
                            .walls
                            .into_iter()
                            .filter_map(|(s, e)| Some((s, e?)))
                        {
                            println!("Add cost: {idx}");
                            passing.entry((wall_start, wall_end)).or_insert(idx);
                        }
                    } else {
                        let max_cost = idx - target_save;
                        println!("MAX COST: {max_cost}");
                        let mut print_pass = passing
                            .values()
                            .filter(|pass| **pass <= max_cost)
                            .map(|val| idx - val)
                            .collect::<Vec<_>>();
                        print_pass.sort();
                        println!("Pass: {:#?}", print_pass);
                        return passing.values().filter(|pass| **pass <= max_cost).count() as u64;
                    }
                } else {
                    visited.insert(next_node.cur_pos, next_node.walls);

                    let new_visits = Direction::all()
                        .into_iter()
                        .flat_map(|dir| next_node.cur_pos.step_dir(dir))
                        .flat_map(|new_pos| {
                            if let Some(is_wall) = self.grid.get(new_pos).copied() {
                                if !next_node.walls.is_empty() || !is_wall {
                                    let walls = if let Some(existing_wall) = &next_node.walls {
                                        (Some(*existing_wall), next_node.wall_end.or(Some(new_pos)))
                                    } else if is_wall {
                                        (Some(next_node.cur_pos), None)
                                    } else {
                                        (None, None)
                                    };
                                    let new_node = CostNode {
                                        cur_pos: new_pos,
                                        wall_start,
                                        wall_end,
                                    };
                                    if !visited.contains(&new_node) {
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
        }

        0
    }
}
