use std::collections::HashMap;

use advent_rust_lib::{grid::RectangleGrid, read::input};

fn part1(grid: &RectangleGrid<bool>) -> u64 {
    grid.positioned_items()
        .filter(|entry| *entry.value)
        .filter(|entry| {
            entry
                .position()
                .surrounding_pos()
                .filter(|surround| *grid.get(*surround).unwrap_or(&false))
                .count()
                < 4
        })
        .count() as u64
}

fn part2(grid: &RectangleGrid<bool>) -> u64 {
    // (pos, [other pos...])
    let around_pos: HashMap<_, _> = grid
        .positioned_items()
        .filter(|entry| *entry.value)
        .map(|entry| {
            (
                entry.position(),
                entry
                    .position()
                    .surrounding_pos()
                    .filter(|surround| *grid.get(*surround).unwrap_or(&false))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    // (pos, num)
    let mut around_pos_counts: HashMap<_, _> = around_pos
        .iter()
        .map(|(pos, around)| (*pos, around.len()))
        .collect();

    let mut total_count = 0;

    // Holds surrounding pos entries.
    let mut to_decrement = Vec::with_capacity(around_pos.len());

    loop {
        around_pos_counts.retain(|entry, count| {
            let remove = *count < 4;
            if remove {
                to_decrement.extend(around_pos.get(entry).unwrap());
                total_count += 1;
            }

            !remove
        });

        // Reached steady state
        if to_decrement.is_empty() {
            break;
        }

        for elem in &to_decrement {
            if let Some(elem) = around_pos_counts.get_mut(elem) {
                *elem -= 1;
            }
        }

        // Keep reusing this vec
        to_decrement.clear();
    }

    total_count
}

// Executes in around 10 ms on my machine.
fn main() {
    let grid = RectangleGrid::try_from_iter(
        input().map(|line| line.chars().map(|c| c == '@').collect::<Vec<_>>()),
    )
    .unwrap();

    println!("Part 1: {}", part1(&grid));
    println!("Part 2: {}", part2(&grid));
}
