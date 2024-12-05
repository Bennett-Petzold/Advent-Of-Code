use std::{
    collections::HashMap,
    env::args,
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::Itertools;

fn main() {
    let (mut left, mut right): (Vec<_>, Vec<_>) =
        BufReader::new(File::open(args().nth(1).unwrap()).unwrap())
            .lines()
            .map(|line| {
                let line = line.unwrap();
                let mut line = line.split_whitespace();
                let left = line.next().unwrap();
                let right = line.next().unwrap();
                (
                    str::parse::<usize>(left).unwrap(),
                    str::parse::<usize>(right).unwrap(),
                )
            })
            .unzip();
    left.sort_unstable();
    right.sort_unstable();

    part_1(left.clone(), right.clone());
    part_2(left, right);
}

fn part_1(left: Vec<usize>, right: Vec<usize>) {
    let pairs = left.iter().zip(right.iter());
    let total_diff: usize = pairs.map(|(l, r)| l.abs_diff(*r)).sum();
    println!("{total_diff}");
}

fn part_2(left: Vec<usize>, right: Vec<usize>) {
    let left_occurs = left.into_iter().chunk_by(|x| *x);
    let left_occurs = left_occurs.into_iter().map(|(key, group)| ListOccurrence {
        num: key,
        repeats: group.count(),
    });
    let right_occurs: HashMap<usize, usize> = right
        .into_iter()
        .chunk_by(|x| *x)
        .into_iter()
        .map(|(key, group)| (key, group.count()))
        .collect();

    let mut sum = 0;
    for left_inst in left_occurs {
        if let Some(right_inst) = right_occurs.get(&left_inst.num) {
            sum += left_inst.apply(*right_inst);
        }
    }
    println!("{sum}")
}

#[derive(Debug)]
struct ListOccurrence {
    pub num: usize,
    pub repeats: usize,
}

impl ListOccurrence {
    pub fn apply(&self, count: usize) -> usize {
        self.num * self.repeats * count
    }
}
