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
    let price = garden.total_price_alt();
    println!("{price}");
}

fn part_2(garden: &Garden) {
    let price = garden.total_price_fencing_alt();
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
        let mut count = 0;

        for entry in self.grid.positioned_items() {
            let (area, edges) =
                self.traverse_region((entry.position(), *entry.value), &mut traversed);
            count += area * edges;
        }

        count
    }

    pub fn total_price_alt(&self) -> u64 {
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

    const ONE_OFFSET: Pos2D = Pos2D::new(1, 1);

    fn traverse_region_fencing(
        &self,
        entry: (Pos2D, char),
        traversed: &mut HashSet<Pos2D>,
        edges: &mut Vec<Pos2D>,
    ) -> Vec<Pos2D> {
        if !traversed.contains(&entry.0) {
            traversed.insert(entry.0);
            let mut area = vec![entry.0];

            for dir in Direction::all() {
                if let Some(pos) = entry.0.step_dir(dir) {
                    if let Some(value) = self.grid.get(pos) {
                        if *value == entry.1 {
                            let mut inner_area =
                                self.traverse_region_fencing((pos, *value), traversed, edges);
                            area.append(&mut inner_area);
                        } else {
                            edges.push(pos + Self::ONE_OFFSET);
                        }
                    } else {
                        edges.push(pos + Self::ONE_OFFSET);
                    }
                } else {
                    edges.push(
                        (entry.0 + Self::ONE_OFFSET)
                            .step_dir(dir)
                            .expect("(z + 1) - 1 is always >= 0 for positive z"),
                    );
                }
            }

            area
        } else {
            Vec::new()
        }
    }

    pub fn total_price_fencing(&self) -> u64 {
        let mut traversed = HashSet::new();
        let mut edges = Vec::new();
        let mut count = 0;

        for entry in self.grid.positioned_items() {
            edges.clear();

            let mut area = self.traverse_region_fencing(
                (entry.position(), *entry.value),
                &mut traversed,
                &mut edges,
            );

            // All area, ordered by highest and then by leftmost
            area.sort_unstable_by(Pos2D::order_top_left);

            // All edges, ordered by highest and then by leftmost
            edges.sort_unstable_by(Pos2D::order_top_left);
            edges.dedup();

            let mut lines = 0;

            while let Some(bottom_rightmost) = edges.pop() {
                // Account for right and down origins (Position is offset by +(1, 1))
                let mut sides = [Direction::Left, Direction::Up]
                    .into_iter()
                    .flat_map(|dir| bottom_rightmost.step_dir(dir))
                    .filter(|pos| {
                        area.binary_search_by(|inner| Pos2D::order_top_left(inner, pos))
                            .is_ok()
                    })
                    .count() as u64;

                // Account for left origin
                if (bottom_rightmost - Self::ONE_OFFSET)
                    .and_then(|pos| pos.left())
                    .and_then(|pos| {
                        area.binary_search_by(|inner| Pos2D::order_top_left(inner, &pos))
                            .ok()
                    })
                    .is_some()
                {
                    sides += 1;
                }

                // Account for up origin
                if (bottom_rightmost - Self::ONE_OFFSET)
                    .and_then(|pos| pos.up())
                    .and_then(|pos| {
                        area.binary_search_by(|inner| Pos2D::order_top_left(inner, &pos))
                            .ok()
                    })
                    .is_some()
                {
                    sides += 1;
                }

                let mut removed_edges = Vec::new();

                // Remove the line extending left
                {
                    let mut leftmost = bottom_rightmost;
                    while let Some(left_pos) = leftmost.left() {
                        if let Some(next) = edges.last() {
                            if *next == left_pos {
                                leftmost = left_pos;
                                removed_edges.push(edges.pop().unwrap());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                // Remove the line extending up
                {
                    let mut upmost = bottom_rightmost;
                    while let Some(up_pos) = upmost.up() {
                        if let Ok(idx) =
                            edges.binary_search_by(|edge| Pos2D::order_top_left(edge, &up_pos))
                        {
                            upmost = up_pos;
                            removed_edges.push(edges.remove(idx));
                        } else {
                            break;
                        }
                    }
                }

                // An edge must be touching at least one inner plant
                debug_assert_ne!(sides, 0, "{}: {bottom_rightmost}", entry.value);

                println!(
                    "{} edge: {bottom_rightmost} -> {sides} => {:?}",
                    entry.value, removed_edges
                );
                lines += sides;
            }

            println!("{} area, sides: {}, {lines}", entry.value, area.len());

            count += (area.len() as u64) * lines;
        }

        count
    }

    fn print_region(vals: &HashSet<Pos2D>, global_edges: &mut i64) {
        let x_limit = vals
            .iter()
            .map(|pos| pos.x)
            .reduce(|acc, x| if x < acc { acc } else { x })
            .unwrap()
            + 1;
        let y_limit = vals
            .iter()
            .map(|pos| pos.y)
            .reduce(|acc, y| if y < acc { acc } else { y })
            .unwrap()
            + 1;

        for y in 0..y_limit {
            let mut line = vec![" "; x_limit].into_boxed_slice();
            for x in vals.iter().filter(|pos| pos.y == y).map(|pos| pos.x) {
                line[x] = "%";
            }

            println!("{:?}", line);
        }
        println!("Num edges: {global_edges}\n");
    }

    fn traverse_region_fencing_alt(
        &self,
        entry: (Pos2D, char),
        traversed: &mut HashSet<Pos2D>,
        global_edges: &mut i64,
    ) -> (u64, i64) {
        let mut edges = 0;
        let mut area = 1;

        for dir in Direction::all() {
            if let Some(pos) = entry.0.step_dir(dir) {
                if let Some(value) = self.grid.get(pos) {
                    if *value == entry.1 && !traversed.contains(&pos) {
                        traversed.insert(pos);

                        let num_extra_touching = Direction::all()
                            .into_iter()
                            .filter(|x| *x != dir.reverse())
                            .flat_map(|secondary_dir| pos.step_dir(secondary_dir))
                            .filter(|secondary_pos| traversed.contains(secondary_pos))
                            .count();
                        println!("num extra touching {pos}: {num_extra_touching}");
                        let mut ordered_traversed: Vec<_> = traversed.iter().collect();
                        ordered_traversed.sort_unstable();
                        //println!("traversed: {:#?}", ordered_traversed);
                        let diagonals = match dir.reverse() {
                            Direction::Up => [pos.up_left(), pos.up_right()],
                            Direction::Down => [pos.down_left(), pos.down_right()],
                            Direction::Left => [pos.up_left(), pos.down_left()],
                            Direction::Right => [pos.down_right(), pos.up_right()],
                        };
                        match num_extra_touching {
                            // Only touching one side
                            0 => {
                                /*
                                let diagonals = [
                                    pos.up_left(),
                                    pos.up_right(),
                                    pos.down_left(),
                                    pos.down_right(),
                                ];
                                */

                                //let diag_vec: Vec<_> = diagonals.iter().flatten().collect();
                                //println!("Diagonals: {:#?}", diag_vec);
                                //
                                let diagonals = diagonals
                                    .into_iter()
                                    .flatten()
                                    .filter(|secondary_pos| traversed.contains(secondary_pos));

                                let count = diagonals.count();

                                if count > 0 {
                                    println!("Added {}", count * 2);
                                }

                                edges += 2 * (count as i64);
                                *global_edges += 2 * (count as i64);
                            }
                            // Touching two or three sides, closed out a single touch
                            1 | 2 => {
                                let diagonals = [
                                    pos.up_left(),
                                    pos.up_right(),
                                    pos.down_left(),
                                    pos.down_right(),
                                ]
                                .into_iter()
                                .flatten()
                                .filter(|secondary_pos| traversed.contains(secondary_pos));

                                if diagonals.count() < 2 {
                                    println!("Subtracted 2");
                                    edges -= 2;
                                    *global_edges -= 2
                                }
                            }
                            // Filled in a square if its touching 4 sides
                            3 => {
                                println!("Subtracted 4");
                                edges -= 4;
                                *global_edges -= 4
                            }
                            _ => unreachable!("Max number of sides is 4"),
                        }

                        Self::print_region(traversed, global_edges);

                        let (inner_area, inner_edges) = self.traverse_region_fencing_alt(
                            (pos, *value),
                            traversed,
                            global_edges,
                        );

                        area += inner_area;
                        edges += inner_edges;
                    }
                }
            }
        }

        (area, edges)
    }

    pub fn total_price_fencing_alt(&self) -> u64 {
        let mut traversed = HashSet::new();
        let mut region = HashSet::new();
        let mut count = 0;

        for entry in self.grid.positioned_items() {
            if !traversed.contains(&entry.position()) {
                region.insert(entry.position());
                let mut global_edges = 4;
                let (area, edges) = self.traverse_region_fencing_alt(
                    (entry.position(), *entry.value),
                    &mut region,
                    &mut global_edges,
                );

                traversed.extend(region.drain());

                debug_assert!(
                    edges >= 0,
                    "{:?}: num extra edges ({edges}) must be nonnegative",
                    entry
                );

                // Base case: square with 4 edges
                // The return from traversal is the modification to this
                const SQUARE_EDGES: u64 = 4;
                let edges = SQUARE_EDGES + (edges as u64);
                println!("Num edges for {:?}: {edges}", entry);

                count += area * edges;
            }
        }

        count
    }
}
