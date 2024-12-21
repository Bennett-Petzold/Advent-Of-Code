use std::{
    cmp::{min, Reverse},
    collections::{BinaryHeap, HashMap, HashSet},
    env::args,
};

use advent_rust_lib::{
    direction::Direction,
    grid::{Pos2D, RectangleGrid},
    ll::{RevLinkedNode, RevLinkedNodeInternal},
    read::filtered_input,
};

fn main() {
    let target_save: usize = str::parse(&args().nth(1).unwrap()).unwrap();
    let track = Track::from_input_iter(filtered_input(&[2])).unwrap();

    part_1(&track, target_save);
}

fn part_1(track: &Track, target_save: usize) {
    println!("{}", track.num_valid_cheats(target_save));
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
struct CanonNode {
    cur_pos: Pos2D,
    cost: u64,
    node: RevLinkedNode,
}

impl PartialEq for CanonNode {
    fn eq(&self, other: &Self) -> bool {
        (self.cur_pos == other.cur_pos) && (self.cost == other.cost)
    }
}

impl Eq for CanonNode {}

impl PartialOrd for CanonNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CanonNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.cur_pos.cmp(&other.cur_pos))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheatNode {
    cur_pos: Pos2D,
    cost: usize,
}

impl PartialOrd for CheatNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CheatNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.cur_pos.cmp(&other.cur_pos))
    }
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

    pub fn canon_path(&self) -> RevLinkedNode {
        let mut visited = HashSet::new();
        let mut to_visit = BinaryHeap::from([Reverse(CanonNode {
            cur_pos: self.start,
            cost: 0,
            node: RevLinkedNodeInternal::push(RevLinkedNodeInternal::new(), self.start),
        })]);

        loop {
            let next_node = to_visit.pop().expect("always at least one").0;

            if next_node.cur_pos == self.end {
                return next_node.node;
            } else {
                visited.insert(next_node.cur_pos);

                let new_visits = Direction::all()
                    .into_iter()
                    .flat_map(|dir| next_node.cur_pos.step_dir(dir))
                    .flat_map(|new_pos| {
                        if self.grid.get(new_pos).copied() == Some(false) {
                            let new_node = CanonNode {
                                cur_pos: new_pos,
                                cost: next_node.cost + 1,
                                node: RevLinkedNodeInternal::push(next_node.node.clone(), new_pos),
                            };
                            if !visited.contains(&new_node.cur_pos) {
                                Some(Reverse(new_node))
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

    pub fn num_valid_cheats(&self, target_save: usize) -> usize {
        // Enumerated with value 0 at cost 0, sorted with value n at cost n
        let (in_order_canon, sorted_canon) = {
            let mut canon: Vec<_> = RevLinkedNodeInternal::iter(self.canon_path()).collect();

            let sorted_canon: HashMap<_, _> = canon
                .iter()
                .cloned()
                .enumerate()
                .map(|(idx, pos)| (pos, idx))
                .collect();

            canon.reverse();

            (canon, sorted_canon)
        };

        let canon_cost = in_order_canon.len() - 1;
        let target_cutoff = canon_cost - target_save;
        println!("Target cutoff: {target_cutoff}");

        let mut num_passing = 0;
        let mut counts = HashMap::new();
        for idx in 0..target_cutoff {
            let shortest_with_cheat = self.eval_cheat(&sorted_canon, &in_order_canon[..=idx], idx);
            if let Some(valid) = canon_cost.checked_sub(shortest_with_cheat) {
                *counts.entry(valid).or_insert(0) += 1;
            }

            if shortest_with_cheat <= target_cutoff {
                num_passing += 1;
            }
        }
        println!("{:#?}", counts);
        num_passing
    }

    pub fn eval_cheat(
        &self,
        sorted_canon: &HashMap<Pos2D, usize>,
        eval_visited: &[Pos2D],
        cost: usize,
    ) -> usize {
        if let Some(eval_pos) = eval_visited.last() {
            if cost == 20 {
                println!("POS: {eval_pos}");
            }

            let mut visited = HashSet::new();
            visited.extend(eval_visited);

            let mut to_visit = BinaryHeap::from_iter(
                Direction::all()
                    .into_iter()
                    .flat_map(|dir| eval_pos.step_dir(dir))
                    .filter(|new_pos| self.grid.get(*new_pos).cloned() == Some(true))
                    .map(|new_pos| {
                        Reverse(CheatNode {
                            cur_pos: new_pos,
                            cost: cost + 1,
                        })
                    }),
            );

            if cost == 20 {
                println!("VISIT: {:#?}", to_visit);
            }

            let mut min_cost = usize::MAX;

            while let Some(next_node) = to_visit.pop() {
                let next_node = next_node.0;

                if visited.contains(&next_node.cur_pos) {
                    continue;
                }

                // Always end exec on end touch
                if next_node.cur_pos == self.end {
                    return min(next_node.cost, min_cost);
                // Close out early with a known min path when touching canon path
                } else if let Some(canon_cost) = sorted_canon.get(&next_node.cur_pos) {
                    let adjusted_cost = next_node.cost + canon_cost;

                    if cost == 20 {
                        println!("ALIGN: {} + {canon_cost} = {adjusted_cost}", next_node.cost);
                    }

                    min_cost = min(min_cost, adjusted_cost);
                    if cost == 20 {
                        println!("NEW MIN: {min_cost}");
                    }
                } else {
                    visited.insert(next_node.cur_pos);

                    let new_visits = Direction::all()
                        .into_iter()
                        .flat_map(|dir| next_node.cur_pos.step_dir(dir))
                        .flat_map(|new_pos| {
                            if self.grid.get(new_pos).copied() == Some(false) {
                                let new_node = CheatNode {
                                    cur_pos: new_pos,
                                    cost: next_node.cost + 1,
                                };
                                if !visited.contains(&new_node.cur_pos) {
                                    Some(Reverse(new_node))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        });

                    let new_visits: Vec<_> = new_visits.collect();
                    if cost == 20 {
                        println!("SUBMIT: {:#?}", new_visits);
                    }
                    to_visit.extend(new_visits);
                }
            }

            if cost == 20 {
                println!("EXIT NORMALLY: {min_cost}");
            }

            min_cost
        } else {
            usize::MAX
        }
    }
}
