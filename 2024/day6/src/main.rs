use std::collections::HashMap;

use advent_rust_lib::{grid::Pos2D, read::input};

fn main() {
    let map = GuardMap::from_input(input()).unwrap();
    part_1(&map);
    part_2(&map);
}

fn part_1(map: &GuardMap) {
    let count = map.count_unique_until_exit();
    println!("Part 1: {count}")
}

fn part_2(map: &GuardMap) {
    let count = map.num_possible_blockages();
    println!("Part 2: {count}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardDir {
    Up,
    Left,
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Guard {
    pos: Pos2D,
    dir: GuardDir,
}

impl Guard {
    pub fn new(pos: Pos2D) -> Self {
        Self {
            pos,
            dir: GuardDir::Up,
        }
    }

    pub fn next(&self) -> Option<Self> {
        let pos = match self.dir {
            GuardDir::Up => self.pos.up(),
            GuardDir::Left => self.pos.left(),
            GuardDir::Right => self.pos.right(),
            GuardDir::Down => self.pos.down(),
        }?;

        Some(Self { pos, dir: self.dir })
    }

    pub fn rotate(&mut self) {
        self.dir = match self.dir {
            GuardDir::Up => GuardDir::Right,
            GuardDir::Right => GuardDir::Down,
            GuardDir::Down => GuardDir::Left,
            GuardDir::Left => GuardDir::Up,
        }
    }

    pub fn pos(&self) -> Pos2D {
        self.pos
    }

    pub fn dir(&self) -> GuardDir {
        self.dir
    }
}

#[derive(Debug)]
pub struct GuardMap {
    // (y, [x, ...]). x vector is sorted.
    obstacles: HashMap<usize, Vec<usize>>,
    guard: Guard,
    max_x: usize,
    max_y: usize,
}

#[cfg(not(feature = "print"))]
type SpecLoopReturn = bool;
#[cfg(feature = "print")]
type SpecLoopReturn = Option<Vec<Pos2D>>;

impl GuardMap {
    pub fn from_input<S, I>(lines: I) -> Option<Self>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let mut guard = None;
        let mut obstacles = HashMap::new();
        let mut max_x = 0;
        let mut max_y = 0;

        for (outer_idx, line) in lines.enumerate() {
            if max_y == 0 {
                max_x = line.as_ref().len();
            }
            max_y = outer_idx;

            if guard.is_none() {
                if let Some(idx) = line.as_ref().find('^') {
                    guard = Some(Guard::new(Pos2D::new(idx, outer_idx)));
                }
            }

            let mut x_vals: Vec<_> = line
                .as_ref()
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == '#')
                .map(|(idx, _)| idx)
                .collect();
            if !x_vals.is_empty() {
                x_vals.sort_unstable();
                obstacles.insert(outer_idx, x_vals);
            }
        }

        let guard = guard?;
        let max_y = max_y + 1;

        Some(Self {
            obstacles,
            guard,
            max_x,
            max_y,
        })
    }

    pub fn count_unique_until_exit(&self) -> usize {
        let mut guard = self.guard;
        let mut unique = vec![guard.pos()];

        #[cfg(feature = "print")]
        let mut print_unique = vec![guard.pos()];

        // While still in bounds
        while guard.pos().x < self.max_x && guard.pos().y < self.max_y {
            if let Some(next_guard) = guard.next() {
                // Checking for dupes here speeds along running repeated paths
                if let Err(insert_pos) = unique.binary_search(&next_guard.pos()) {
                    // Check if next position is an invalid tile
                    if self
                        .obstacles
                        .get(&next_guard.pos().y)
                        .map(|blocked_x| blocked_x.binary_search(&next_guard.pos().x).is_ok())
                        == Some(true)
                    {
                        // Rotate 90 and rerun
                        guard.rotate();
                        continue;
                    } else {
                        // New tile is unique, insert and set as the new val
                        unique.insert(insert_pos, next_guard.pos());
                        guard = next_guard;

                        #[cfg(feature = "print")]
                        print_unique.push(next_guard.pos());
                    }
                } else {
                    // New tile is not unique, set as the new val
                    guard = next_guard;

                    #[cfg(feature = "print")]
                    print_unique.push(next_guard.pos());
                }
            } else {
                break;
            }
        }

        #[cfg(feature = "print")]
        {
            if guard.pos().x >= self.max_x || guard.pos().y >= self.max_y {
                guard.rotate();
                guard.rotate();
                guard = guard.next().unwrap();
                guard.rotate();
                guard.rotate();
            }
            let mut print_vec_deduped = Vec::with_capacity(print_unique.len());
            for pos in print_unique.into_iter().rev() {
                if !print_vec_deduped.contains(&pos) {
                    print_vec_deduped.push(pos);
                }
            }

            self.print_map(
                &guard,
                print_vec_deduped.into_iter().rev(),
                None,
                std::iter::empty(),
            );
        }

        unique.len().saturating_sub(1)
    }

    fn speculate_loop(
        &self,
        guard: &Guard,
        next_guard: &Guard,
        // MUST be sorted
        unique: &[Guard],
    ) -> SpecLoopReturn {
        let block_pos = next_guard.pos();

        // Speculate on current position being rotated by 90
        let mut speculative_guard = *guard;
        speculative_guard.rotate();

        if let Ok(start_idx) = unique.binary_search_by_key(&speculative_guard.pos(), |x| x.pos()) {
            for idx in (0..start_idx).rev() {
                if unique[idx].pos() == speculative_guard.pos() {
                    if unique[idx] == speculative_guard {
                        #[cfg(not(feature = "print"))]
                        return true;
                        #[cfg(feature = "print")]
                        return Some(Vec::new());
                    }
                } else {
                    break;
                }
            }

            for idx in start_idx..unique.len() {
                if unique[idx].pos() == speculative_guard.pos() {
                    if unique[idx] == speculative_guard {
                        #[cfg(not(feature = "print"))]
                        return true;
                        #[cfg(feature = "print")]
                        return Some(Vec::new());
                    }
                } else {
                    break;
                }
            }
        }

        let mut spec_unique = vec![speculative_guard];

        #[cfg(feature = "print")]
        let mut spec_unique_print = vec![speculative_guard.pos()];

        while let Some(next_spec) = speculative_guard.next() {
            if next_spec.pos().x >= self.max_x || next_spec.pos().y >= self.max_y {
                #[cfg(not(feature = "print"))]
                return false;
                #[cfg(feature = "print")]
                return None;
            }

            if let Err(insert_pos) = spec_unique.binary_search(&next_spec) {
                // Check if next position is an invalid tile
                if (self
                    .obstacles
                    .get(&next_spec.pos().y)
                    .map(|blocked_x| blocked_x.binary_search(&next_spec.pos().x).is_ok())
                    == Some(true))
                    || next_spec.pos() == block_pos
                {
                    // Rotate 90 and rerun
                    speculative_guard.rotate();
                } else {
                    // Duplicate with primary looping
                    if let Ok(start_idx) =
                        unique.binary_search_by_key(&next_spec.pos(), |x| x.pos())
                    {
                        for idx in (0..start_idx).rev() {
                            if unique[idx].pos() == next_spec.pos() {
                                if unique[idx] == next_spec {
                                    #[cfg(not(feature = "print"))]
                                    return true;
                                    #[cfg(feature = "print")]
                                    return Some(spec_unique_print);
                                }
                            } else {
                                break;
                            }
                        }

                        for idx in start_idx..unique.len() {
                            if unique[idx].pos() == next_spec.pos() {
                                if unique[idx] == next_spec {
                                    #[cfg(not(feature = "print"))]
                                    return true;
                                    #[cfg(feature = "print")]
                                    return Some(spec_unique_print);
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    // New tile is unique, insert and set as the new val
                    spec_unique.insert(insert_pos, next_spec);
                    speculative_guard = next_spec;

                    #[cfg(feature = "print")]
                    spec_unique_print.push(next_spec.pos());
                }
            } else {
                // Repeat in the speculative loop
                #[cfg(not(feature = "print"))]
                return true;
                #[cfg(feature = "print")]
                return Some(spec_unique_print);
            }
        }

        #[cfg(not(feature = "print"))]
        return false;
        #[cfg(feature = "print")]
        return None;
    }

    pub fn num_possible_blockages(&self) -> usize {
        let mut guard = self.guard;
        let mut unique = vec![guard];
        let mut num_blocks = 0;

        #[cfg(feature = "print")]
        let mut print_unique = vec![guard.pos()];

        // While still in bounds
        while guard.pos().x < self.max_x && guard.pos().y < self.max_y {
            if let Some(next_guard) = guard.next() {
                // Checking for dupes here speeds along running repeated paths
                if let Err(insert_pos) = unique.binary_search_by_key(&next_guard.pos(), |x| x.pos())
                {
                    // Check if next position is an invalid tile
                    if self
                        .obstacles
                        .get(&next_guard.pos().y)
                        .map(|blocked_x| blocked_x.binary_search(&next_guard.pos().x).is_ok())
                        == Some(true)
                    {
                        // Rotate 90 and rerun
                        guard.rotate();
                    } else {
                        let loops = self.speculate_loop(&guard, &next_guard, &unique);

                        // New tile is unique, insert and set as the new val
                        unique.insert(insert_pos, next_guard);
                        guard = next_guard;

                        #[cfg(feature = "print")]
                        print_unique.push(next_guard.pos());

                        #[cfg(not(feature = "print"))]
                        if loops {
                            num_blocks += 1;
                        }

                        #[cfg(feature = "print")]
                        if let Some(spec_unique_print) = loops {
                            num_blocks += 1;

                            let mut print_vec_deduped = Vec::with_capacity(print_unique.len());
                            for pos in std::mem::take(&mut print_unique).into_iter().rev() {
                                if !print_vec_deduped.contains(&pos) {
                                    print_vec_deduped.push(pos);
                                }
                            }
                            print_unique = print_vec_deduped.into_iter().rev().collect();

                            self.print_map(
                                &guard,
                                print_unique.iter().cloned(),
                                Some(next_guard.pos()),
                                spec_unique_print.into_iter(),
                            );
                        }
                    }
                } else {
                    // New tile is not unique, set as the new val
                    guard = next_guard;

                    #[cfg(feature = "print")]
                    print_unique.push(next_guard.pos());
                }
            } else {
                break;
            }
        }

        num_blocks
    }

    /// Debug print
    #[cfg(feature = "print")]
    pub fn print_map<I0, I1>(
        &self,
        guard: &Guard,
        unique: I0,
        blockage: Option<Pos2D>,
        speculative: I1,
    ) where
        I0: ExactSizeIterator<Item = Pos2D>,
        I1: ExactSizeIterator<Item = Pos2D>,
    {
        use colored::Colorize;
        use colorgrad::Gradient;

        {
            let mut tiles = vec![vec![" ".to_string(); self.max_x]; self.max_y];

            let guard_char = match guard.dir() {
                GuardDir::Up => "^",
                GuardDir::Right => ">",
                GuardDir::Down => "v",
                GuardDir::Left => "<",
            };

            let colors: Vec<_> = colorgrad::preset::turbo()
                .colors(unique.len() + (unique.len() / 5))
                .into_iter()
                .skip(unique.len() / 5)
                .collect();

            unique
                .into_iter()
                .zip(colors.iter())
                .for_each(|(pos, color)| {
                    if let Some(tile) = tiles.get_mut(pos.y).and_then(|line| line.get_mut(pos.x)) {
                        let color = color.to_rgba8();
                        *tile = "•".truecolor(color[0], color[1], color[2]).to_string();
                    }
                });

            {
                let colors_speculative: Vec<_> = colorgrad::preset::turbo()
                    .colors(speculative.len() + (speculative.len() / 5))
                    .into_iter()
                    .skip(speculative.len() / 5)
                    .collect();

                speculative
                    .into_iter()
                    .zip(colors_speculative.iter())
                    .for_each(|(pos, color)| {
                        if let Some(tile) =
                            tiles.get_mut(pos.y).and_then(|line| line.get_mut(pos.x))
                        {
                            let color = color.to_rgba8();
                            *tile = "$".truecolor(color[0], color[1], color[2]).to_string();
                        }
                    });
            }

            {
                let color = colors[0].to_rgba8();
                tiles[self.guard.pos().y][self.guard.pos().x] =
                    "^".truecolor(color[0], color[1], color[2]).to_string();
            }

            tiles[guard.pos().y][guard.pos().x] = guard_char.to_string();
            if let Some(blockage) = blockage {
                tiles[blockage.y][blockage.x] = "O".to_string();
            }

            self.obstacles.iter().for_each(|(y, x_vals)| {
                let line = &mut tiles[*y];
                x_vals.iter().for_each(|x| line[*x] = "#".to_string());
            });

            println!();
            println!("{}", "--".repeat(self.max_x));
            for tile_line in tiles {
                println!(
                    "{}",
                    tile_line
                        .into_iter()
                        .flat_map(|x| [x, " ".to_string()])
                        .collect::<String>()
                );
            }
            println!("{}", "--".repeat(self.max_x));
        }
    }
}
