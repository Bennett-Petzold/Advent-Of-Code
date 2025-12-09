use std::{
    cmp::min,
    ops::{Add, Mul},
};

use advent_rust_lib::read::input;

#[derive(Debug, Clone)]
pub struct Problem {
    items: Vec<u64>,
    op: fn(u64, u64) -> u64,
}

impl Problem {
    fn resolve(&self) -> u64 {
        self.items.iter().copied().reduce(self.op).unwrap_or(0)
    }
}

fn parse_input() -> Vec<Problem> {
    let mut input = input();
    if let Some(first_line) = input.next() {
        let mut problems: Vec<_> = first_line
            .split_whitespace()
            .map(|item| Problem {
                items: vec![item.parse().unwrap()],
                op: u64::add,
            })
            .collect();

        for line in input {
            problems.iter_mut().zip(line.split_whitespace()).for_each(
                |(problem, item)| match item {
                    "*" => problem.op = u64::mul,
                    "+" => problem.op = u64::add,
                    x => {
                        if let Ok(num) = x.parse() {
                            problem.items.push(num)
                        } else {
                            panic!("WTF: {x}")
                        }
                    }
                },
            );
        }

        problems
    } else {
        Vec::new()
    }
}

fn parse_rot_input() -> Vec<Problem> {
    let mut input: Vec<_> = input().collect();
    if let Some((operators, rest)) = input.split_last() {
        let operators = operators.split_whitespace().map(|item| match item {
            "*" => u64::mul,
            "+" => u64::add,
            x => {
                panic!("WTF: {x}")
            }
        });

        let mut rotated: Vec<String> = Vec::new();

        rest.iter().for_each(|line| {
            line.chars().enumerate().for_each(|(pos, c)| {
                if let Some(rot_line) = rotated.get_mut(pos) {
                    rot_line.push(c);
                } else {
                    rotated.push(c.to_string());
                }
            });
        });

        let rotated_chunks = rotated.split(|line| line.trim().is_empty());

        rotated_chunks
            .zip(operators)
            .map(|(line, op)| Problem {
                items: line
                    .iter()
                    .map(|item| item.trim().parse::<u64>().unwrap())
                    .collect(),
                op,
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn main() {
    println!(
        "Part 1: {}",
        parse_input()
            .into_iter()
            .map(|problem| problem.resolve())
            .sum::<u64>()
    );

    println!(
        "Part 2: {:#?}",
        parse_rot_input()
            .into_iter()
            .map(|problem| problem.resolve())
            .sum::<u64>()
    );
}
