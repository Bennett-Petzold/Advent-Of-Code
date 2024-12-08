use std::collections::HashMap;

use advent_rust_lib::read::input;
use itertools::Itertools;

fn main() {
    let mut input_iter = input();
    let mapping: HashMap<_, Vec<_>> = input_iter
        .by_ref()
        .take_while(|line| !line.trim().is_empty())
        .map(|line| {
            let mut splits = line.split('|');
            (
                str::parse::<u64>(splits.next().unwrap()).unwrap(),
                str::parse::<u64>(splits.next().unwrap()).unwrap(),
            )
        })
        .sorted_by(|(_, y0), (_, y1)| y0.cmp(y1))
        .chunk_by(|(_, y)| *y)
        .into_iter()
        .map(|(group, vals)| (group, vals.into_iter().map(|(x, _)| x).collect()))
        .collect();

    let pages: Vec<_> = input_iter
        .map(|line| {
            line.split(',')
                .map(|x| str::parse::<u64>(x).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    part_1(&mapping, &pages);
    part_2(&mapping, &pages);
}

fn part_1(mapping: &HashMap<u64, Vec<u64>>, pages: &[Vec<u64>]) {
    let valid_pages = pages.iter().filter(|line| {
        let mut previous = Vec::new();
        for item in line.iter() {
            if let Some(prev_reqs) = mapping.get(item) {
                if prev_reqs
                    .iter()
                    .any(|req| line.contains(req) && !previous.contains(req))
                {
                    return false;
                }
            }

            previous.push(*item);
        }
        true
    });

    let count: u64 = valid_pages.map(|line| line[line.len() / 2]).sum();
    println!("{count}");
}

fn part_2(mapping: &HashMap<u64, Vec<u64>>, pages: &[Vec<u64>]) {
    let invalid_pages = pages.iter().filter(|line| {
        let mut previous = Vec::new();
        for item in line.iter() {
            if let Some(prev_reqs) = mapping.get(item) {
                if prev_reqs
                    .iter()
                    .any(|req| line.contains(req) && !previous.contains(req))
                {
                    return true;
                }
            }

            previous.push(*item);
        }
        false
    });

    let reordered_pages = invalid_pages.map(|line| reorder(mapping, line.clone()));

    let count: u64 = reordered_pages.map(|line| line[line.len() / 2]).sum();
    println!("{count}");
}

fn reorder(mapping: &HashMap<u64, Vec<u64>>, mut line: Vec<u64>) -> Vec<u64> {
    let mut previous = Vec::new();
    for (idx, item) in line.iter().enumerate() {
        if let Some(prev_reqs) = mapping.get(item) {
            for req in prev_reqs {
                if line.contains(req) && !previous.contains(req) {
                    line.remove(line.iter().position(|x| x == req).unwrap());
                    line.insert(idx, *req);
                    return reorder(mapping, line);
                }
            }
        }

        previous.push(*item);
    }
    line
}
