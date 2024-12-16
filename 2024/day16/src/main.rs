use std::{
    cmp::{min, Reverse},
    collections::{BinaryHeap, HashMap, HashSet},
};

use advent_rust_lib::{
    direction::Direction,
    grid::{Pos2D, RectangleGrid},
    read::input,
};

fn main() {
    let maze = Maze::from_input_lines(input()).unwrap();
    part_1(&maze);
    part_2(&maze);
}

fn part_1(maze: &Maze) {
    println!("{}", maze.min_score());
}

fn part_2(maze: &Maze) {
    println!("{}", maze.num_tiles_on_best_paths());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Reindeer {
    pub facing: Direction,
    pub pos: Pos2D,
}

#[derive(Debug)]
struct Maze {
    // True when a wall, false otherwise
    grid: RectangleGrid<bool>,
    start: Reindeer,
    end: Pos2D,
}

impl Maze {
    pub fn from_input_lines<S, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut start = None;
        let mut end = None;

        let iter = iter
            .into_iter()
            .enumerate()
            // Find start character
            .inspect(|(y_idx, line)| {
                if start.is_none() {
                    if let Some(x_idx) = line.as_ref().find('S') {
                        start = Some(Reindeer {
                            facing: Direction::Left,
                            pos: Pos2D::new(x_idx, *y_idx),
                        })
                    }
                }
            })
            // Find end character
            .inspect(|(y_idx, line)| {
                if end.is_none() {
                    if let Some(x_idx) = line.as_ref().find('E') {
                        end = Some(Pos2D::new(x_idx, *y_idx))
                    }
                }
            })
            .map(|(_, line)| line.as_ref().chars().map(|c| c == '#').collect::<Vec<_>>());

        let grid = RectangleGrid::try_from_iter(iter).ok()?;

        Some(Self {
            grid,
            start: start?,
            end: end?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReindeerTraversal {
    pub reindeer: Reindeer,
    pub cost: u64,
}

impl PartialOrd for ReindeerTraversal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReindeerTraversal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.reindeer.cmp(&other.reindeer))
    }
}

impl Maze {
    /// Returns the stepped reindeer, if valid
    fn step_reindeer(&self, reindeer: Reindeer) -> Option<Reindeer> {
        let new_reindeer = Reindeer {
            pos: reindeer.pos.step_dir(reindeer.facing)?,
            facing: reindeer.facing,
        };

        if !(*self.grid.get(new_reindeer.pos)?) {
            Some(new_reindeer)
        } else {
            None
        }
    }

    fn min_score(&self) -> u64 {
        let mut visited = HashSet::new();
        let mut to_visit = BinaryHeap::from([Reverse(ReindeerTraversal {
            reindeer: self.start,
            cost: 0,
        })]);

        const TURN_COST: u64 = 1000;
        const STEP_COST: u64 = 1;

        while to_visit
            .peek()
            .expect("always at least one item to visit")
            .0
            .reindeer
            .pos
            != self.end
        {
            let element = to_visit.pop().expect("always at least one item to visit").0;

            // Skip processing redundant elements.
            if visited.insert(element.reindeer) {
                let clockwise = ReindeerTraversal {
                    reindeer: Reindeer {
                        facing: element.reindeer.facing.clockwise(),
                        pos: element.reindeer.pos,
                    },
                    cost: element.cost + TURN_COST,
                };
                if !visited.contains(&clockwise.reindeer) {
                    to_visit.push(Reverse(clockwise));
                }

                let counter_clockwise = ReindeerTraversal {
                    reindeer: Reindeer {
                        facing: element.reindeer.facing.counter_clockwise(),
                        pos: element.reindeer.pos,
                    },
                    cost: element.cost + TURN_COST,
                };
                if !visited.contains(&counter_clockwise.reindeer) {
                    to_visit.push(Reverse(counter_clockwise));
                }

                if let Some(new_reindeer) = self.step_reindeer(element.reindeer) {
                    if !visited.contains(&new_reindeer) {
                        to_visit.push(Reverse(ReindeerTraversal {
                            reindeer: new_reindeer,
                            cost: element.cost + STEP_COST,
                        }));
                    }
                }
            }
        }

        to_visit
            .peek()
            .expect("always at least one item to visit")
            .0
            .cost
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TrackingReindeerTraversal {
    pub reindeer: Reindeer,
    pub cost: u64,
    pub previous_deer: Vec<ReindeerTraversal>,
}

impl TrackingReindeerTraversal {
    pub fn poses_with_self(&self) -> Vec<ReindeerTraversal> {
        let mut poses = self.previous_deer.clone();
        poses.push(self.without_tracking());
        poses
    }

    pub fn without_tracking(&self) -> ReindeerTraversal {
        ReindeerTraversal {
            reindeer: self.reindeer,
            cost: self.cost,
        }
    }
}

impl PartialOrd for TrackingReindeerTraversal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackingReindeerTraversal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.reindeer.cmp(&other.reindeer))
            .then(self.previous_deer.cmp(&other.previous_deer))
    }
}

impl Maze {
    fn num_tiles_on_best_paths(&self) -> u64 {
        let mut visited = HashMap::new();
        let mut to_visit = BinaryHeap::from([Reverse(TrackingReindeerTraversal {
            reindeer: self.start,
            cost: 0,
            previous_deer: vec![],
        })]);

        const TURN_COST: u64 = 1000;
        const STEP_COST: u64 = 1;

        let mut min_cost = u64::MAX;
        let mut canonical_visits = HashSet::new();

        while to_visit.peek().map(|val| val.0.cost <= min_cost) == Some(true) {
            let element = to_visit
                .pop()
                .expect("Loop condition requires at least one element")
                .0;

            // Terminal position at or below minimum cost
            if element.reindeer.pos == self.end {
                canonical_visits.extend(
                    element
                        .poses_with_self()
                        .into_iter()
                        .map(|traversal| traversal.reindeer),
                );
                min_cost = element.cost;
            } else {
                // Skip processing redundant elements.
                if visited
                    .get(&element.reindeer)
                    .map(|cost| *cost < element.cost)
                    != Some(true)
                {
                    // Update to a lower cost, if applicable
                    let visited_entry = visited.entry(element.reindeer).or_insert(element.cost);
                    *visited_entry = min(*visited_entry, element.cost);

                    if canonical_visits.contains(&element.reindeer) {
                        canonical_visits.extend(
                            element
                                .previous_deer
                                .into_iter()
                                .map(|traversal| traversal.reindeer),
                        );
                    } else {
                        let clockwise = TrackingReindeerTraversal {
                            reindeer: Reindeer {
                                facing: element.reindeer.facing.clockwise(),
                                pos: element.reindeer.pos,
                            },
                            cost: element.cost + TURN_COST,
                            previous_deer: element.poses_with_self(),
                        };
                        to_visit.push(Reverse(clockwise));

                        let counter_clockwise = TrackingReindeerTraversal {
                            reindeer: Reindeer {
                                facing: element.reindeer.facing.counter_clockwise(),
                                pos: element.reindeer.pos,
                            },
                            cost: element.cost + TURN_COST,
                            previous_deer: element.poses_with_self(),
                        };
                        to_visit.push(Reverse(counter_clockwise));

                        if let Some(new_reindeer) = self.step_reindeer(element.reindeer) {
                            to_visit.push(Reverse(TrackingReindeerTraversal {
                                reindeer: new_reindeer,
                                cost: element.cost + STEP_COST,
                                previous_deer: element.poses_with_self(),
                            }));
                        }
                    }
                }
            }
        }

        let mut visit_vec: Vec<_> = canonical_visits.into_iter().map(|deer| deer.pos).collect();
        visit_vec.sort_unstable();
        visit_vec.dedup();
        visit_vec.len() as u64
    }
}
