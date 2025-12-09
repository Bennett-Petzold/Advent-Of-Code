use std::mem;

use advent_rust_lib::{
    grid::{Pos2D, RectangleGrid},
    read::input,
};

fn part1(start: Pos2D, splitter_grid: &RectangleGrid<bool>) -> u64 {
    let mut beams = vec![start.x];
    let mut split_count = 0;

    for line in splitter_grid.lines().skip(start.y + 1) {
        for beam in mem::take(&mut beams) {
            if line[beam] {
                split_count += 1;
                while let Some(left) = beam.checked_sub(1) {
                    if !line[left] {
                        beams.push(left);
                        break;
                    }
                }
                while let Some(right) = beam.checked_add(1)
                    && right < splitter_grid.x_max()
                {
                    if !line[right] {
                        beams.push(right);
                        break;
                    }
                }
            } else {
                beams.push(beam);
            }
        }

        // Clear out duplicates
        beams.sort_unstable();
        beams.dedup();
    }

    split_count
}

#[derive(Debug, Clone, Copy)]
struct Particle {
    pub x: usize,
    pub timelines: u64,
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
    }
}

impl Eq for Particle {}

impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Particle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x)
    }
}

fn part2(start: Pos2D, splitter_grid: &RectangleGrid<bool>) -> u64 {
    let mut beams = vec![Particle {
        x: start.x,
        timelines: 1,
    }];

    for line in splitter_grid.lines().skip(start.y + 1) {
        for beam in mem::take(&mut beams) {
            if line[beam.x] {
                while let Some(left) = beam.x.checked_sub(1) {
                    if !line[left] {
                        beams.push(Particle {
                            x: left,
                            timelines: beam.timelines,
                        });
                        break;
                    }
                }
                while let Some(right) = beam.x.checked_add(1)
                    && right < splitter_grid.x_max()
                {
                    if !line[right] {
                        beams.push(Particle {
                            x: right,
                            timelines: beam.timelines,
                        });
                        break;
                    }
                }
            } else {
                beams.push(beam);
            }
        }

        // Combine duplicates timeline counts
        beams.sort_unstable();
        let mut new_beams = Vec::with_capacity(beams.len());
        if let Some(last) = beams.pop() {
            new_beams.push(last);

            for beam in beams.drain(..).rev() {
                let last_new = new_beams.last_mut().expect("Always initialized");
                if beam.x == last_new.x {
                    last_new.timelines += beam.timelines;
                } else {
                    new_beams.push(beam);
                }
            }
        }
        beams = new_beams;
    }

    beams.into_iter().map(|beam| beam.timelines).sum()
}

fn main() {
    let char_grid =
        RectangleGrid::try_from_iter(input().map(|line| line.chars().collect::<Vec<_>>())).unwrap();

    let start = char_grid
        .positioned_items()
        .find(|pos_item| *pos_item.value == 'S')
        .map(|pos_item| pos_item.position())
        .unwrap();

    // True for '^'
    let splitter_grid = char_grid.map(|entry| *entry.value == '^');

    println!("Part 1: {}", part1(start, &splitter_grid));
    println!("Part 1: {}", part2(start, &splitter_grid));
}
