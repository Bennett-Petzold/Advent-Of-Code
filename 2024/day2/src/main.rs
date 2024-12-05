use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
};

fn main() {
    let reports: Vec<_> = BufReader::new(File::open(args().nth(1).unwrap()).unwrap())
        .lines()
        .map(|line| Report::from_line(line.unwrap()).unwrap())
        .collect();

    let num_valid = reports.iter().map(Report::validate).filter(|x| *x).count();
    println!("{num_valid}");

    let num_valid_one_err = reports
        .iter()
        .map(Report::validate_allow_one_error)
        .filter(|x| *x)
        .count();
    println!("{num_valid_one_err}");
}

#[derive(Debug)]
struct Report {
    levels: Box<[u8]>,
}

impl From<Box<[u8]>> for Report {
    fn from(value: Box<[u8]>) -> Self {
        Self { levels: value }
    }
}

impl From<Vec<u8>> for Report {
    fn from(value: Vec<u8>) -> Self {
        Self {
            levels: value.into_boxed_slice(),
        }
    }
}

impl Report {
    fn from_line<S: AsRef<str>>(line: S) -> Result<Self, ParseIntError> {
        line.as_ref()
            .split_whitespace()
            .map(str::parse::<u8>)
            .collect::<Result<Vec<_>, _>>()
            .map(Self::from)
    }

    fn validate(&self) -> bool {
        validate_slice(&self.levels)
    }

    fn validate_allow_one_error(&self) -> bool {
        if self.levels.len() < 2 {
            true
        } else {
            let cmp_dir = self.levels[0].cmp(&self.levels[1]);
            let mut issue_idx = 0;

            let full_res = self
                .levels
                .iter()
                .zip(self.levels.iter().skip(1))
                .enumerate()
                .all(|(idx, (lhs, rhs))| {
                    let res = (lhs.cmp(rhs) == cmp_dir) && (1..=3).contains(&lhs.abs_diff(*rhs));
                    if !res {
                        issue_idx = idx;
                    }
                    res
                });

            if full_res {
                true
            } else {
                let mut trimmed_levels = self.levels.to_vec();
                trimmed_levels.remove(issue_idx);
                if Self::from(trimmed_levels).validate() {
                    true
                } else if issue_idx < (self.levels.len().saturating_sub(1)) {
                    let mut before_trimmed_levels = self.levels.to_vec();
                    before_trimmed_levels.remove(issue_idx + 1);
                    if Self::from(before_trimmed_levels).validate() {
                        true
                    } else {
                        validate_slice(&self.levels[1..])
                    }
                } else {
                    false
                }
            }
        }
    }
}

fn validate_slice(slice: &[u8]) -> bool {
    if slice.len() < 2 {
        true
    } else {
        let cmp_dir = slice[0].cmp(&slice[1]);

        slice
            .iter()
            .zip(slice.iter().skip(1))
            .all(|(lhs, rhs)| (lhs.cmp(rhs) == cmp_dir) && (1..=3).contains(&lhs.abs_diff(*rhs)))
    }
}
