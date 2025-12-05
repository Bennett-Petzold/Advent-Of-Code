use std::{cmp::Ordering, collections::HashMap};

use advent_rust_lib::{
    grid::{Pos2D, RectangleGrid},
    read::input,
};

// Executes in around 6.4 ms on my machine.
fn main() {
    // Already sorted as expected
    let mut counted_grid: Vec<_> = {
        let raw_grid = RectangleGrid::try_from_iter(
            input().map(|line| line.chars().map(|c| c == '@').collect::<Vec<_>>()),
        )
        .unwrap();

        raw_grid
            .positioned_items()
            .flat_map(|entry| {
                if *entry.value {
                    Some((
                        entry.position(),
                        entry
                            .position()
                            .surrounding_pos()
                            .filter(|around_pos| *raw_grid.get(*around_pos).unwrap_or(&false))
                            .count(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    };

    let original_len = counted_grid.len();

    let mut to_decrement = Vec::with_capacity(original_len);

    let cycle = |to_decrement: &mut Vec<Pos2D>, counted_grid: &mut Vec<(Pos2D, usize)>| {
        fn decrement(to_decrement: &mut Vec<Pos2D>, counted_grid: &mut [(Pos2D, usize)]) {
            to_decrement.sort_unstable();
            let mut counted_grid_iter = counted_grid.iter_mut();
            if let Some(mut counted_grid_item) = counted_grid_iter.next() {
                for dec_pos in to_decrement.drain(..) {
                    while counted_grid_item.0 < dec_pos {
                        if let Some(new_item) = counted_grid_iter.next() {
                            counted_grid_item = new_item;
                        } else {
                            return;
                        }
                    }

                    if counted_grid_item.0 == dec_pos {
                        counted_grid_item.1 -= 1;
                    }
                }
            }
        }

        decrement(to_decrement, counted_grid);

        counted_grid.retain(|(item, count): &(_, _)| {
            let remove = *count < 4;
            if remove {
                to_decrement.extend(item.surrounding_pos());
            }

            !remove
        });
    };

    // Single invoke
    cycle(&mut to_decrement, &mut counted_grid);

    println!("Part 1: {}", original_len - counted_grid.len());

    // Run to steady state
    while !to_decrement.is_empty() {
        cycle(&mut to_decrement, &mut counted_grid);
    }

    println!("Part 2: {}", original_len - counted_grid.len());
}
