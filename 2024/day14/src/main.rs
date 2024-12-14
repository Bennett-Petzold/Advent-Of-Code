use std::{cmp::Ordering, num::ParseIntError, sync::LazyLock};

use advent_rust_lib::read::input;
use regex::Regex;
use thiserror::Error;

fn main() {
    let mut input = input();

    let (width, height): (i64, i64) = {
        let next_input = input.next().unwrap();
        let (width, height) = next_input.split_once(',').unwrap();
        (str::parse(width).unwrap(), str::parse(height).unwrap())
    };

    let robots: Vec<_> = input
        .map(Robot::from_input_line)
        .collect::<Result<_, _>>()
        .unwrap();

    part_1(&robots, width, height);
}

fn part_1(robots: &[Robot], width: i64, height: i64) {
    let (half_width, half_height) = (width / 2, height / 2);

    let mut quad_sums = [0; 4];

    robots
        .iter()
        .map(|robot| robot.step(100, width, height))
        .for_each(|(x, y)| {
            let mut idx = match x.cmp(&half_width) {
                Ordering::Less => 0,
                Ordering::Greater => 1,
                Ordering::Equal => return,
            };

            match y.cmp(&half_height) {
                Ordering::Greater => idx += 2,
                Ordering::Less => (),
                Ordering::Equal => return,
            };

            quad_sums[idx] += 1;
        });

    println!("{}", quad_sums.iter().product::<i64>());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Robot {
    pos: (i64, i64),
    change: (i64, i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RobotErr {
    #[error("No position entry")]
    MissingPosition,
    #[error("No change entry")]
    MissingChange,
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
}

impl Robot {
    pub fn from_input_line<S: AsRef<str>>(line: S) -> Result<Self, RobotErr> {
        static POSITION: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#".=((-|\d)+),((-|\d)+)"#).unwrap());

        let mut position_entries = POSITION.captures_iter(line.as_ref()).map(|cap| {
            let (_, [x, _, y, _]) = cap.extract();
            Ok::<_, ParseIntError>((str::parse(x)?, str::parse(y)?))
        });

        let pos = position_entries.next().ok_or(RobotErr::MissingPosition)??;
        let change = position_entries.next().ok_or(RobotErr::MissingChange)??;

        Ok(Self { pos, change })
    }

    /// Returns final position (x, y) after stepping `times` with wraparound.
    pub fn step(self, times: i64, max_width: i64, max_height: i64) -> (i64, i64) {
        let fit_to_box = |z: i64, max_z: i64| {
            if z.is_negative() {
                let adjusted = max_z - (-z % max_z);
                if adjusted == max_z {
                    0
                } else {
                    adjusted
                }
            } else {
                z % max_z
            }
        };

        let final_x = self.pos.0 + (self.change.0 * times);
        let final_y = self.pos.1 + (self.change.1 * times);

        (
            fit_to_box(final_x, max_width),
            fit_to_box(final_y, max_height),
        )
    }
}
