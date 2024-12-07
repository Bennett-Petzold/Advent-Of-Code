use std::{env::args, fs::read_to_string, num::ParseIntError, sync::LazyLock};

use regex::{Captures, Regex};
use thiserror::Error;

fn main() {
    let input = read_to_string(args().nth(1).unwrap()).unwrap();
    part_1(&input);
    part_2(&input);
}

fn part_1(input: &str) {
    let mul_sum: u64 = get_muls(input)
        .map(|x| x.unwrap())
        .map(|(lhs, rhs)| lhs * rhs)
        .sum();
    println!("{mul_sum}")
}

fn part_2(input: &str) {
    let mut sum = 0;
    let mut active = true;
    for statement in get_statements(input) {
        let statement = statement.unwrap();
        if active {
            match statement {
                Statement::Dont => {
                    active = false;
                }
                Statement::Mul(lhs, rhs) => {
                    sum += lhs * rhs;
                }
                Statement::Do => (),
            }
        } else if statement == Statement::Do {
            active = true;
        }
    }
    println!("{sum}")
}

fn get_muls(to_parse: &str) -> impl Iterator<Item = Result<(u64, u64), ParseIntError>> + use<'_> {
    static MUL_STATEMENT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"mul\((\d+),(\d+)\)"#).unwrap());

    (*MUL_STATEMENT).captures_iter(to_parse).map(|x| {
        let extracted: [_; 2] = x.extract().1;
        Ok((str::parse(extracted[0])?, str::parse(extracted[1])?))
    })
}

fn get_statements(
    to_parse: &str,
) -> impl Iterator<Item = Result<Statement, StatementErr>> + use<'_> {
    static MUL_STATEMENT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(don't\(\))|(do\(\))|(mul\((\d+),(\d+)\))"#).unwrap());

    (*MUL_STATEMENT)
        .captures_iter(to_parse)
        .map(Statement::from_capture)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement {
    Do,
    Dont,
    Mul(u64, u64),
}

#[derive(Debug, Error)]
pub enum StatementErr {
    #[error("An invalid regex result was returned")]
    InvalidRegex,
    #[error("{0}")]
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for StatementErr {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl Statement {
    pub fn from_capture(capture: Captures<'_>) -> Result<Self, StatementErr> {
        let mut subcaptures = capture.iter();
        match subcaptures
            .by_ref()
            .next()
            .flatten()
            .ok_or(StatementErr::InvalidRegex)?
            .as_str()
        {
            "don't()" => Ok(Self::Dont),
            "do()" => Ok(Self::Do),
            x if x.starts_with("mul(") => {
                let mut digits = subcaptures
                    .flatten()
                    .skip(1)
                    .map(|y| str::parse(y.as_str()));
                let lhs = digits.by_ref().next().ok_or(StatementErr::InvalidRegex)??;
                let rhs = digits.next().ok_or(StatementErr::InvalidRegex)??;
                Ok(Self::Mul(lhs, rhs))
            }
            _ => Err(StatementErr::InvalidRegex),
        }
    }
}
