use std::ops::{Add, Sub};

use crate::signed::SignedUsize;

/// Position in a 2D grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos2D {
    pub x: usize,
    pub y: usize,
}

impl Pos2D {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    // -- START Directional calculations -- //

    pub fn down(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_add(1)?))
    }

    pub fn down_right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y.checked_add(1)?))
    }

    pub fn right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y))
    }

    pub fn up_right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y.checked_sub(1)?))
    }

    pub fn up(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_sub(1)?))
    }

    pub fn up_left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y.checked_sub(1)?))
    }

    pub fn left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y))
    }

    pub fn down_left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y.checked_add(1)?))
    }

    // -- END Directional calculations -- //

    pub fn surrounding_pos(&self) -> impl Iterator<Item = Self> {
        [
            Some(Self::new(self.x + 1, self.y)),
            Some(Self::new(self.x + 1, self.y + 1)),
            Some(Self::new(self.x, self.y + 1)),
            // -- negative x -- //
            self.x.checked_sub(1).map(|x| Self::new(x, self.y)),
            self.x.checked_sub(1).map(|x| Self::new(x, self.y + 1)),
            // -- negative y -- //
            self.y.checked_sub(1).map(|y| Self::new(self.x, y)),
            self.y.checked_sub(1).map(|y| Self::new(self.x + 1, y)),
            // -- negative x and y -- //
            self.x
                .checked_sub(1)
                .and_then(|x| self.y.checked_sub(1).map(|y| Self::new(x, y))),
        ]
        .into_iter()
        .flatten()
    }

    pub fn surrounding_lines(&self) -> impl Iterator<Item = SurroundingLineIter> + use<'_> {
        [
            Self::down,
            Self::down_right,
            Self::right,
            Self::up_right,
            Self::up,
            Self::up_left,
            Self::left,
            Self::down_left,
        ]
        .into_iter()
        .map(|func| SurroundingLineIter::new(*self, func))
    }

    pub fn get_arr_char<S, A>(&self, arr: A) -> Option<char>
    where
        S: AsRef<str>,
        A: AsRef<[S]>,
    {
        arr.as_ref().get(self.y)?.as_ref().chars().nth(self.x)
    }
}

impl From<(usize, usize)> for Pos2D {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Add for Pos2D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Pos2D {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Some(Self::new(
            self.x.checked_sub(rhs.x)?,
            self.y.checked_sub(rhs.y)?,
        ))
    }
}

impl Pos2D {
    // Returns a position with the absolute difference.
    pub fn abs_diff(self, rhs: Self) -> Self {
        Self {
            x: self.x.abs_diff(rhs.x),
            y: self.y.abs_diff(rhs.y),
        }
    }

    // Returns a position flipped around `center_point`.
    pub fn flip(mut self, center_point: Self) -> Option<Self> {
        let x_diff = (SignedUsize::from(self.x) - SignedUsize::from(center_point.x))?;
        let y_diff = (SignedUsize::from(self.y) - SignedUsize::from(center_point.y))?;

        self.x = (center_point.x - x_diff)?;
        self.y = (center_point.y - y_diff)?;

        Some(self)
    }

    // Returns a position rotated once around `center_point`.
    fn rot_once(mut self, center_point: Self) -> Option<Self> {
        if self == center_point {
            return Some(self);
        }

        let x_diff = (SignedUsize::from(self.x) - SignedUsize::from(center_point.x))?;
        let y_diff = (SignedUsize::from(self.y) - SignedUsize::from(center_point.y))?;

        if x_diff.is_zero() {
            self.x = (center_point.x + y_diff)?;
            self.y = center_point.y;
        } else if y_diff.is_zero() {
            self.y = (center_point.y - x_diff)?;
            self.x = center_point.x;
        } else {
            match (x_diff.sign(), y_diff.sign()) {
                // Top left corner
                (true, true) => self.y = (center_point.y - y_diff)?,
                // Bottom left corner
                (true, false) => self.x = (center_point.x - x_diff)?,
                // Bottom right corner
                (false, true) => self.y = (center_point.y + y_diff)?,
                // Top right corner
                (false, false) => self.x = (center_point.x + x_diff)?,
            }
        }

        Some(self)
    }

    // Returns a position rotated 90 degrees `num_rotations` times around `center_point`.
    pub fn rotate_clockwise_90(self, center_point: Self, num_rotations: u8) -> Option<Self> {
        match num_rotations % 4 {
            0 => Some(self),
            1 => self.rot_once(center_point),
            2 => self.flip(center_point),
            3 => {
                let this = self.flip(center_point)?;
                this.rot_once(center_point)
            }
            // Guaranteed impossible values by the modulo
            4..=u8::MAX => unreachable!(),
        }
    }

    // Returns all 90 degree rotations in order of 0, 90, 180, and 270.
    pub fn all_90_clockwise_rotations(self, center_point: Self) -> [Option<Self>; 4] {
        [
            Some(self),
            self.rot_once(center_point),
            self.flip(center_point),
            self.rot_once(center_point)
                .and_then(|this| this.rot_once(center_point)),
        ]
    }
}

#[derive(Debug)]
pub struct SurroundingLineIter {
    cur_pos: Pos2D,
    change: fn(&Pos2D) -> Option<Pos2D>,
}

impl SurroundingLineIter {
    pub fn new(pos: Pos2D, change: fn(&Pos2D) -> Option<Pos2D>) -> Self {
        Self {
            cur_pos: pos,
            change,
        }
    }
}

impl Iterator for SurroundingLineIter {
    type Item = Pos2D;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_pos = (self.change)(&self.cur_pos)?;
        Some(self.cur_pos)
    }
}
