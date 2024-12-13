use std::collections::HashSet;

use advent_rust_lib::{
    direction::Direction,
    grid::{NonRectangleInput, Pos2D, RectangleGrid},
    read::input,
};

fn main() {
    let garden = Garden::from_str_iter(input()).unwrap();
    part_1(&garden);
    part_2(&garden);
}

fn part_1(garden: &Garden) {
    let price = garden.total_price();
    println!("{price}");
}

fn part_2(garden: &Garden) {
    let price = garden.total_price_fencing();
    println!("{price}");
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
struct Garden {
    pub grid: RectangleGrid<char>,
}

impl Garden {
    pub fn from_str_iter<I, S>(iter: I) -> Result<Self, NonRectangleInput>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Ok(Self {
            grid: RectangleGrid::try_from_iter(
                iter.into_iter()
                    .map(|line| line.as_ref().chars().collect::<Vec<_>>()),
            )?,
        })
    }

    fn traverse_region(&self, entry: (Pos2D, char), traversed: &mut HashSet<Pos2D>) -> (u64, u64) {
        if !traversed.contains(&entry.0) {
            traversed.insert(entry.0);
            let mut area = 1;
            let mut edges = 0;

            for dir in Direction::all() {
                if let Some(pos) = entry.0.step_dir(dir) {
                    if let Some(value) = self.grid.get(pos) {
                        if *value == entry.1 {
                            let (inner_area, inner_edges) =
                                self.traverse_region((pos, *value), traversed);
                            area += inner_area;
                            edges += inner_edges;
                        } else {
                            edges += 1;
                        }
                    } else {
                        edges += 1;
                    }
                } else {
                    edges += 1;
                }
            }

            (area, edges)
        } else {
            (0, 0)
        }
    }

    pub fn total_price(&self) -> u64 {
        let mut traversed = HashSet::new();
        let mut region = HashSet::new();
        let mut count = 0;

        for entry in self.grid.positioned_items() {
            if !traversed.contains(&entry.position()) {
                let (area, edges) =
                    self.traverse_region((entry.position(), *entry.value), &mut region);
                traversed.extend(region.drain());

                count += area * edges;
            }
        }

        count
    }

    const OFFSET: Pos2D = Pos2D::new(1, 1);

    fn traverse_region_corners(
        &self,
        entry: (Pos2D, char),
        traversed: &mut HashSet<Pos2D>,
        corners: &mut Vec<Pos2D>,
    ) -> u64 {
        if !traversed.contains(&entry.0) {
            traversed.insert(entry.0);

            // Add all diagonal corners (with an offset to include -1 coordinates)
            let offset_pos = entry.0 + Self::OFFSET;
            let offset_diagonals = [
                offset_pos.up_left(),
                offset_pos.down_left(),
                offset_pos.up_right(),
                offset_pos.down_right(),
            ]
            .into_iter()
            .flatten();
            corners.extend(offset_diagonals);

            let mut area = 1;

            for dir in Direction::all() {
                if let Some(pos) = entry.0.step_dir(dir) {
                    if let Some(value) = self.grid.get(pos) {
                        if *value == entry.1 {
                            let inner_area =
                                self.traverse_region_corners((pos, *value), traversed, corners);
                            area += inner_area;
                        }
                    }
                }
            }

            area
        } else {
            0
        }
    }

    pub fn total_price_fencing(&self) -> u64 {
        let mut traversed = HashSet::new();
        let mut region = HashSet::new();
        let mut count = 0;

        for entry in self.grid.positioned_items() {
            if !traversed.contains(&entry.position()) {
                let (area, _) = self.traverse_region((entry.position(), *entry.value), &mut region);

                let offset_region: HashSet<_> =
                    region.iter().map(|pos| *pos + Pos2D::new(1, 1)).collect();

                let corner_counts = offset_region.iter().map(|corner_candidate| {
                    // (pos, check_dir_0, check_dir_1)
                    let diagonals = [
                        (
                            corner_candidate.up_left().unwrap(),
                            Direction::Down,
                            Direction::Right,
                        ),
                        (
                            corner_candidate.up_right().unwrap(),
                            Direction::Down,
                            Direction::Left,
                        ),
                        (
                            corner_candidate.down_left().unwrap(),
                            Direction::Up,
                            Direction::Right,
                        ),
                        (
                            corner_candidate.down_right().unwrap(),
                            Direction::Up,
                            Direction::Left,
                        ),
                    ]
                    .into_iter();

                    let non_region_diagonals =
                        diagonals.filter(|(pos, ..)| !offset_region.contains(pos));

                    let correct_touching_diagonals =
                        non_region_diagonals.filter(|(pos, check_dir_0, check_dir_1)| {
                            let num_touching = [*check_dir_0, *check_dir_1]
                                .into_iter()
                                .flat_map(|dir| pos.step_dir(dir))
                                .filter(|touching| offset_region.contains(touching));

                            // 0 is an outer edge, 2 is an inner edge
                            [0, 2].contains(&num_touching.count())
                        });

                    correct_touching_diagonals.count() as u64
                });

                // Only checked from the bottom
                let special_inner_corners = region.iter().map(|pos| {
                    let mut special_corners = 0;

                    if let Some(corner) = pos.up_left() {
                        if region.contains(&corner)
                            && !region.contains(&pos.left().unwrap())
                            && !region.contains(&pos.up().unwrap())
                        {
                            special_corners += 1
                        }
                    }

                    if let Some(corner) = pos.up_right() {
                        if region.contains(&corner)
                            && !region.contains(&pos.right().unwrap())
                            && !region.contains(&pos.up().unwrap())
                        {
                            special_corners += 1
                        }
                    }

                    special_corners
                });

                let num_corners: u64 =
                    corner_counts.sum::<u64>() + (special_inner_corners.sum::<u64>() * 2);

                // Add to total traversal
                traversed.extend(region.drain());

                count += area * num_corners;
            }
        }

        count
    }
}
