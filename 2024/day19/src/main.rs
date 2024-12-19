use std::collections::{HashMap, HashSet};

use advent_rust_lib::read::input;

fn main() {
    let mut input = input();
    let patterns: HashSet<String> = input
        .by_ref()
        .next()
        .unwrap()
        .split(',')
        .map(|x| x.trim().to_string())
        .collect();

    // Skip empty line
    input.by_ref().next();

    let designs: Vec<String> = input.collect();

    part_1(&patterns, &designs);
    part_2(&patterns, &designs);
}

fn part_1(patterns: &HashSet<String>, designs: &[String]) {
    let num_possible = designs
        .iter()
        .filter(|design| find_pattern(patterns, design))
        .count();
    println!("{num_possible}");
}

fn part_2(patterns: &HashSet<String>, designs: &[String]) {
    let count_satisfying: u64 = designs
        .iter()
        .map(|design| count_satisfying(patterns, design))
        .sum();
    println!("{count_satisfying}");
}

fn find_pattern(patterns: &HashSet<String>, design: &str) -> bool {
    if patterns.contains(design) {
        true
    } else {
        // Skip full slice
        for end_idx in (0..design.len()).rev() {
            let sub_design = &design[0..end_idx];
            let remaining_design = &design[end_idx..];
            if patterns.contains(sub_design) && find_pattern(patterns, remaining_design) {
                return true;
            }
        }

        // None of the slices resolved into a match
        false
    }
}

fn count_satisfying(patterns: &HashSet<String>, design: &str) -> u64 {
    let mut valid_end_map = HashMap::new();

    // Matches starting from end of string
    let valid_start_to_end_iter = (0..design.len() + 1).rev().map(|start| {
        (
            start,
            (start..design.len() + 1).filter(move |end| patterns.contains(&design[start..*end])),
        )
    });

    for (start, ends) in valid_start_to_end_iter {
        let terminating_ends = ends.map(|end| {
            if end == design.len() {
                1
            } else if let Some(val) = valid_end_map.get(&end) {
                *val
            } else {
                0
            }
        });
        let value = terminating_ends.sum();
        if value > 0 {
            valid_end_map.insert(start, value);
        }
    }

    //println!("VALID END MAP: {:#?}", valid_end_map);
    valid_end_map.get(&0).cloned().unwrap_or(0)
}
