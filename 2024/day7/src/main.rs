use std::cmp::Ordering;

use advent_rust_lib::read::input;

fn main() {
    let equations: Vec<_> = input().map(|line| Equation::new(line).unwrap()).collect();
    part_1(&equations);
    part_2(&equations);
}

fn part_1(equations: &[Equation]) {
    let sum: u64 = equations
        .iter()
        .filter(|eq| eq.validate())
        .map(|eq| eq.value())
        .sum();
    println!("{sum}")
}

fn part_2(equations: &[Equation]) {
    let sum: u64 = equations
        .iter()
        .filter(|eq| eq.validate_with_concats())
        .map(|eq| eq.value())
        .sum();
    println!("{sum}")
}

#[derive(Debug)]
struct Equation {
    value: u64,
    args: Vec<u64>,
}

impl Equation {
    pub fn value(&self) -> u64 {
        self.value
    }

    #[allow(dead_code)]
    pub fn args(&self) -> &[u64] {
        &self.args
    }
}

impl Equation {
    pub fn new<S: AsRef<str>>(line: S) -> Option<Self> {
        let (value, args) = line.as_ref().split_once(':')?;
        let value = value.parse().ok()?;

        #[allow(clippy::trim_split_whitespace)]
        let args = args
            .trim()
            .split_whitespace()
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()
            .ok()?;

        Some(Self { value, args })
    }

    /// The repeated validation logic that may recurse.
    ///
    /// * `args`: ALWAYS 1 or more elements
    fn inner_validate(mut cur_val: u64, args: &[u64]) -> bool {
        let target = args[0];

        for (idx, arg) in args[1..].iter().rev().enumerate() {
            // If evenly divisible, assess against the remaining idx
            if (cur_val % arg) == 0
                && Self::inner_validate(cur_val / arg, &args[..args.len() - (idx + 1)])
            {
                return true;
            }

            match cur_val.cmp(arg) {
                Ordering::Greater => cur_val -= arg,
                // Overflow
                Ordering::Equal | Ordering::Less => return false,
            }
        }

        cur_val == target
    }

    pub fn validate(&self) -> bool {
        match self.args.len() {
            0 => false,
            1 => self.value == self.args[0],
            // x > 1
            _ => Self::inner_validate(self.value, &self.args),
        }
    }

    /// The repeated validation logic that may recurse.
    ///
    /// * `last_arg`: Previous argument
    /// * `args`: ALWAYS 1 or more elements
    fn inner_validate_with_concats(mut cur_val: u64, args: &[u64]) -> bool {
        let target = args[0];

        for (idx, arg) in args[1..].iter().rev().enumerate() {
            // If evenly divisible, assess against the remaining idx
            if (cur_val % arg) == 0
                && Self::inner_validate_with_concats(cur_val / arg, &args[..args.len() - (idx + 1)])
            {
                return true;
            }

            // Test the || operator if arg is the last digits of cur_val
            {
                let arg_num_digits = Self::num_digits(*arg);

                if Self::last_digits_match(cur_val, *arg, arg_num_digits)
                    && Self::inner_validate_with_concats(
                        // Remove arg from end of cur_val
                        cur_val / (10_u64.pow(arg_num_digits)),
                        &args[..args.len() - (idx + 1)],
                    )
                {
                    return true;
                }
            }

            match cur_val.cmp(arg) {
                Ordering::Greater => cur_val -= arg,
                // Overflow
                Ordering::Equal | Ordering::Less => return false,
            }
        }

        cur_val == target
    }

    /// Returns the number of digits in base 10.
    fn num_digits(val: u64) -> u32 {
        val.ilog10() + 1
    }

    /// Checks if the last `num_digits` of each number are equal.
    ///
    /// Returns `true` when `num_digits` == 0.
    fn last_digits_match(mut lhs: u64, mut rhs: u64, num_digits: u32) -> bool {
        for _ in 0..num_digits {
            let equal_last_digit = (lhs % 10) == (rhs % 10);

            if equal_last_digit {
                // Next step
                lhs /= 10;
                rhs /= 10;
            } else {
                return false;
            }
        }

        true
    }

    /// return = [lhs digits] | [rhs digits]
    #[expect(unused)]
    fn concat(lhs: u64, rhs: u64) -> u64 {
        let rhs_num_digits = Self::num_digits(rhs);
        let lhs_shift = 10_u64.pow(rhs_num_digits);

        (lhs * lhs_shift) + rhs
    }

    pub fn validate_with_concats(&self) -> bool {
        match self.args.len() {
            0 => false,
            1 => self.value == self.args[0],
            // x > 1
            _ => Self::inner_validate_with_concats(self.value, &self.args),
        }
    }
}
