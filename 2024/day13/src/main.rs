use std::{cmp::min, num::ParseIntError, sync::LazyLock};

use advent_rust_lib::read::input;
use num::Integer;
use regex::Regex;
use thiserror::Error;

fn main() {
    let machines = {
        let mut input = input();
        let mut machines = Vec::new();
        while let Some(line_0) = input.next() {
            machines.push(
                ClawMachine::from_three_lines(std::iter::once(line_0).chain(input.by_ref()))
                    .unwrap(),
            );
            // Discard the empty line
            let _ = input.next();
        }
        machines
    };

    part_1(&machines);
    part_2(&machines);
}

fn part_1(machines: &[ClawMachine]) {
    let sum: u64 = machines
        .iter()
        .map(|machine| machine.min_token_presses())
        .sum();
    println!("{sum}");
}

fn part_2(machines: &[ClawMachine]) {
    let sum: u64 = machines
        .iter()
        .cloned()
        .map(|mut machine| {
            machine.prize.0 += 10000000000000;
            machine.prize.1 += 10000000000000;

            machine.min_token_presses()
        })
        .sum();
    println!("{sum}");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ClawMachine {
    button_a: (i64, i64),
    button_b: (i64, i64),
    prize: (i64, i64),
}

#[derive(Debug, Error)]
pub enum ClawMachineParseError {
    #[error("The iterator does not have enough lines")]
    EmptyIter,
    #[error("A line from the iterator is only partial")]
    PartialIter,
    #[error("Could not parse a dimension")]
    ParseIntErr(#[from] ParseIntError),
}

impl ClawMachine {
    /// Takes three lines from `iter` in the claw machine formatting.
    pub fn from_three_lines<I, S>(iter: I) -> Result<Self, ClawMachineParseError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        static BUTTON: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"Button .: X\+(\d+), Y\+(\d+)"#).unwrap());
        static PRIZE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"Prize: X=(\d+), Y=(\d+)"#).unwrap());

        let mut iter = iter.into_iter();

        let mut line = |regex: &Regex| {
            let iter_next = iter.next().ok_or(ClawMachineParseError::EmptyIter)?;
            let vals = regex
                .captures(iter_next.as_ref())
                .ok_or(ClawMachineParseError::PartialIter)?;
            // Skip the full match
            let mut vals_iter = vals.iter().skip(1);

            let x = str::parse(
                vals_iter
                    .next()
                    .flatten()
                    .ok_or(ClawMachineParseError::PartialIter)?
                    .as_str(),
            )?;
            let y = str::parse(
                vals_iter
                    .next()
                    .flatten()
                    .ok_or(ClawMachineParseError::PartialIter)?
                    .as_str(),
            )?;
            Ok::<_, ClawMachineParseError>((x, y))
        };

        let button_a = line(&BUTTON)?;
        let button_b = line(&BUTTON)?;
        let prize = line(&PRIZE)?;

        Ok(Self {
            button_a,
            button_b,
            prize,
        })
    }
}

impl ClawMachine {
    /// Look for an integer solution to a 2x2 system of equations.
    /// A non-integer solution means no number of presses is possible.
    ///
    /// Returns Option<(lhs_coeff, rhs_coeff)>
    fn solve_two_by_two(
        lhs: (i64, i64),
        rhs: (i64, i64),
        target: (i64, i64),
    ) -> Option<(u64, u64)> {
        // x * x_coeff = y * y_coeff
        let lhs_lcm = lhs.0.lcm(&lhs.1);
        let lhs_x_coeff = -(lhs_lcm / lhs.0);
        let lhs_y_coeff = lhs_lcm / lhs.1;

        // [lhs * lhs.0 + rhs * rhs.0] = target.0
        // [lhs * lhs.1 + rhs * rhs.1] = target.1
        //
        // Eliminate lhs by multiplying coefficients
        //
        // [rhs * rhs.0 * lhs_x_coeff] = target.0 * lhs_x_coeff
        // [rhs * rhs.1 * lhs_y_coeff] = target.1 * lhs_y_coeff
        //
        // Consolidate to one equation
        // rhs * [(rhs.0 * lhs_x_coeff) + (rhs.1 * lhs_y_coeff)] = ...
        let rhs_x = rhs.0 * lhs_x_coeff;
        let rhs_y = rhs.1 * lhs_y_coeff;
        let rhs_total = rhs_x + rhs_y;

        // ... = (target.0 * lhs_x_coeff) + (target.1 * lhs_y_coeff)
        let target_x = target.0 * lhs_x_coeff;
        let target_y = target.1 * lhs_y_coeff;
        let target_total = target_x + target_y;

        // Finish solve for rhs
        let b = target_total / rhs_total;
        // B integer check, no extra cost to modulo on most architectures
        let b_is_integer = (target_total % (rhs_total)) == 0;

        // Arbitrarily use the top equation to solve for lhs
        let lhs_top = target.0 - (rhs.0 * b);
        let a = lhs_top / lhs.0;
        // A integer check, no extra cost to modulo on most architectures
        let a_is_integer = (lhs_top % lhs.0) == 0;

        // Only integer coefficients are valid
        if b_is_integer && a_is_integer {
            let a = a as u64;
            let b = b as u64;
            Some((a, b))
        } else {
            None
        }
    }

    /// Minimum number of token presses to reach the prize.
    ///
    /// Zero indicates either a == prize, b == prize, or no combination of
    /// presses reaches the prize.
    pub fn min_token_presses(&self) -> u64 {
        // Only ever one solution to the system of equations.
        let a_solve = Self::solve_two_by_two(self.button_a, self.button_b, self.prize)
            .map(|solve| (solve.0 * 3) + solve.1);

        a_solve.unwrap_or(0)
    }
}
