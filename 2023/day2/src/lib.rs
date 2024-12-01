use std::str::FromStr;

use anyhow::{anyhow, bail};
use derive_getters::Getters;
use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, EnumIter)]
pub enum Color {
    Blue,
    Red,
    Green,
}

impl FromStr for Color {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blue" => Ok(Self::Blue),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            x => bail!("{x} is not a valid color"),
        }
    }
}

#[derive(Debug, Getters, PartialEq, Eq)]
pub struct Cube {
    color: Color,
    count: u32,
}

impl FromStr for Cube {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (count, color) = s
            .trim()
            .split(' ')
            .collect_tuple()
            .ok_or(anyhow!("\"{s}\" is not \"COLOR NUM\""))?;
        Ok(Self {
            color: Color::from_str(color)?,
            count: u32::from_str(count)?,
        })
    }
}

#[derive(Debug, Getters)]
pub struct Game {
    id: u32,
    rounds: Vec<Vec<Cube>>,
}

impl FromStr for Game {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if &s[0..5] != "Game " {
            bail!("{s} does not start with \"Game \"");
        };

        let s = &s[5..];
        let colon = s
            .find(':')
            .ok_or(anyhow!("{s} does not contain a \":\" character"))?;

        let id = u32::from_str(&s[0..colon])?;
        let rounds = s[colon + 1..]
            .split(';')
            .map(|p| {
                p.split(',')
                    .map(Cube::from_str)
                    .collect::<anyhow::Result<Vec<_>>>()
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self { id, rounds })
    }
}

impl Game {
    pub fn min_possible(&self, color: &Color) -> Option<u32> {
        self.rounds
            .iter()
            .flatten()
            .filter(|x| x.color() == color)
            .map(|x| x.count())
            .max()
            .copied()
    }

    pub fn mins_within(&self, conditions: &[(Color, u32)]) -> bool {
        !conditions.iter().any(|cond| {
            self.min_possible(&cond.0)
                .is_some_and(|count| count > cond.1)
        })
    }

    pub fn mins(&self) -> Vec<u32> {
        Color::iter()
            .filter_map(|color| self.min_possible(&color))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_game() {
        let game =
            Game::from_str("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green").unwrap();
        assert_eq!(*game.id(), 1);
        assert_eq!(
            game.rounds()[0],
            vec![
                Cube {
                    color: Color::Blue,
                    count: 3
                },
                Cube {
                    color: Color::Red,
                    count: 4
                }
            ]
        );
    }

    #[test]
    fn sum_valid_games() {
        let conditions = vec![(Color::Red, 12), (Color::Green, 13), (Color::Blue, 14)];

        let res: u32 = vec![
            "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green",
            "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
            "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
            "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
            "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green",
        ]
        .into_iter()
        .map(Game::from_str)
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap()
        .into_iter()
        .filter(|game| game.mins_within(&conditions))
        .map(|game| *game.id())
        .sum();

        assert_eq!(res, 8);
    }

    #[test]
    fn game_min_powers() {
        let res: u32 = vec![
            "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green",
            "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
            "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
            "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
            "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green",
        ]
        .into_iter()
        .map(Game::from_str)
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap()
        .into_iter()
        .map(|game| game.mins())
        .map(|out_vec| out_vec.into_iter().reduce(|acc, x| acc * x).unwrap())
        .sum();

        assert_eq!(res, 2286);
    }
}
