use std::{fmt::Display, ops::Add};

use crate::grid::Pos2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}

impl Direction {
    pub fn all() -> [Self; 4] {
        [Self::Up, Self::Down, Self::Left, Self::Right]
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Pos2D {
    pub fn step_dir(self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::Up => self.up(),
            Direction::Down => self.down(),
            Direction::Left => self.left(),
            Direction::Right => self.right(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirectionSet {
    Empty,
    One([Direction; 1]),
    Two([Direction; 2]),
    Three([Direction; 3]),
    Four,
}

impl Iterator for DirectionSet {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DirectionSet::Empty => None,
            DirectionSet::One(dir) => {
                let ret = Some(dir[0]);
                *self = DirectionSet::Empty;
                ret
            }
            DirectionSet::Two(dirs) => {
                let ret = Some(dirs[1]);
                *self = DirectionSet::One([dirs[0]]);
                ret
            }
            DirectionSet::Three(dirs) => {
                let ret = Some(dirs[2]);
                *self = DirectionSet::Two([dirs[0], dirs[1]]);
                ret
            }
            DirectionSet::Four => {
                let dirs = Direction::all();
                let ret = Some(dirs[3]);
                *self = DirectionSet::Three([dirs[0], dirs[1], dirs[2]]);
                ret
            }
        }
    }
}

impl ExactSizeIterator for DirectionSet {
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::One(_) => 1,
            Self::Two(_) => 2,
            Self::Three(_) => 3,
            Self::Four => 4,
        }
    }
}

impl DirectionSet {
    pub fn reverse(self) -> Self {
        match self {
            DirectionSet::Empty => DirectionSet::Four,
            DirectionSet::One([x]) => DirectionSet::Three(match x {
                Direction::Up => [Direction::Down, Direction::Left, Direction::Right],
                Direction::Down => [Direction::Up, Direction::Left, Direction::Right],
                Direction::Left => [Direction::Up, Direction::Down, Direction::Right],
                Direction::Right => [Direction::Up, Direction::Down, Direction::Left],
            }),
            DirectionSet::Two([x, y]) => DirectionSet::Two([x.reverse(), y.reverse()]),
            DirectionSet::Three(vals) => DirectionSet::One([Direction::all()
                .into_iter()
                .find(|dir| vals.contains(dir))
                .expect("There is always one remaining direction")]),
            DirectionSet::Four => DirectionSet::Empty,
        }
    }

    pub fn all() -> Self {
        DirectionSet::Four
    }

    pub fn single(dir: Direction) -> Self {
        DirectionSet::One([dir])
    }
}

impl Add<Direction> for DirectionSet {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        if self == DirectionSet::Four || self.into_iter().any(|x| x == rhs) {
            self
        } else {
            match self {
                DirectionSet::Empty => DirectionSet::One([rhs]),
                DirectionSet::One([x]) => DirectionSet::Two([x, rhs]),
                DirectionSet::Two([x, y]) => DirectionSet::Three([x, y, rhs]),
                DirectionSet::Three(_) => DirectionSet::Four,
                // Covered by earlier check
                DirectionSet::Four => unreachable!(),
            }
        }
    }
}

impl Add for DirectionSet {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            DirectionSet::Empty => self,
            DirectionSet::One([rhs_val]) => self + rhs_val,
            x => x.into_iter().fold(self, |acc, dir| acc + dir),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SafeDirectionSet(DirectionSet);

impl Iterator for SafeDirectionSet {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl ExactSizeIterator for SafeDirectionSet {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl SafeDirectionSet {
    pub fn reverse(self) -> Self {
        Self(self.0.reverse())
    }

    pub fn all() -> Self {
        Self(DirectionSet::all())
    }

    pub fn single(dir: Direction) -> Self {
        Self(DirectionSet::single(dir))
    }
}

impl Add<Direction> for SafeDirectionSet {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl Add for SafeDirectionSet {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}
