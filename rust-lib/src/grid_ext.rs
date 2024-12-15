use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Sub},
};

use num::{CheckedAdd, CheckedSub, Integer, Signed};

use crate::grid::Pos2D;

/// Position in a 2D grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos2DExt<N> {
    pub x: N,
    pub y: N,
}

impl<N: Integer + Display> Display for Pos2DExt<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<N> Pos2DExt<N> {
    pub const fn new(x: N, y: N) -> Self {
        Self { x, y }
    }
}

impl<N> Pos2DExt<N>
where
    N: Integer + Copy + CheckedAdd + CheckedSub,
    usize: TryFrom<N>,
{
    // -- START Directional calculations -- //

    pub fn down(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_add(&N::one())?))
    }

    pub fn down_right(&self) -> Option<Self> {
        Some(Self::new(
            self.x.checked_add(&N::one())?,
            self.y.checked_add(&N::one())?,
        ))
    }

    pub fn right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(&N::one())?, self.y))
    }

    pub fn up_right(&self) -> Option<Self> {
        Some(Self::new(
            self.x.checked_add(&N::one())?,
            self.y.checked_sub(&N::one())?,
        ))
    }

    pub fn up(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_sub(&N::one())?))
    }

    pub fn up_left(&self) -> Option<Self> {
        Some(Self::new(
            self.x.checked_sub(&N::one())?,
            self.y.checked_sub(&N::one())?,
        ))
    }

    pub fn left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(&N::one())?, self.y))
    }

    pub fn down_left(&self) -> Option<Self> {
        Some(Self::new(
            self.x.checked_sub(&N::one())?,
            self.y.checked_add(&N::one())?,
        ))
    }

    // -- END Directional calculations -- //

    pub fn surrounding_pos(&self) -> impl DoubleEndedIterator<Item = Self> {
        [
            Some(Self::new(self.x + N::one(), self.y)),
            Some(Self::new(self.x + N::one(), self.y + N::one())),
            Some(Self::new(self.x, self.y + N::one())),
            // -- negative x -- //
            self.x.checked_sub(&N::one()).map(|x| Self::new(x, self.y)),
            self.x
                .checked_sub(&N::one())
                .map(|x| Self::new(x, self.y + N::one())),
            // -- negative y -- //
            self.y.checked_sub(&N::one()).map(|y| Self::new(self.x, y)),
            self.y
                .checked_sub(&N::one())
                .map(|y| Self::new(self.x + N::one(), y)),
            // -- negative x and y -- //
            self.x
                .checked_sub(&N::one())
                .and_then(|x| self.y.checked_sub(&N::one()).map(|y| Self::new(x, y))),
        ]
        .into_iter()
        .flatten()
    }

    /*
    pub fn surrounding_lines(&self) -> impl ArrayIter<SurroundingLineIter> + use<'_> {
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

    pub fn repeated_step<F>(&self, step: F) -> StepIter<F>
    where
        F: FnMut(Self) -> Option<Self>,
    {
        StepIter::new(*self, step)
    }
    */

    pub fn get_arr_char<S, A>(&self, arr: A) -> Option<char>
    where
        S: AsRef<str>,
        A: AsRef<[S]>,
    {
        arr.as_ref()
            .get(usize::try_from(self.y).ok()?)?
            .as_ref()
            .chars()
            .nth(usize::try_from(self.x).ok()?)
    }

    /// All edges, ordered by highest and then by leftmost
    ///
    /// e.g. (0 0), (0 1), (1 0), (1 1)
    pub fn order_top_left(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.y.cmp(&rhs.y).then(lhs.x.cmp(&rhs.x))
    }

    /// All edges, ordered by leftmost and then by highest
    ///
    /// e.g. (0 0), (1 0), (0 1), (1 1)
    pub fn order_left_top(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.x.cmp(&rhs.x).then(lhs.y.cmp(&rhs.y))
    }
}

impl<N> From<(N, N)> for Pos2DExt<N> {
    fn from(value: (N, N)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<N: Add<Output = N>> Add for Pos2DExt<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<N: CheckedSub> Sub for Pos2DExt<N> {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Some(Self::new(
            self.x.checked_sub(&rhs.x)?,
            self.y.checked_sub(&rhs.y)?,
        ))
    }
}

impl<N> Pos2DExt<N>
where
    N: Signed + Copy + Sub<Output = N>,
{
    /// Returns a position with the absolute difference.
    pub fn abs_diff(self, rhs: Self) -> Self {
        let part_abs_diff = |lhs, rhs| {
            let res: N = lhs - rhs;
            if res.is_negative() {
                -res
            } else {
                res
            }
        };

        Self {
            x: part_abs_diff(self.x, rhs.x),
            y: part_abs_diff(self.y, rhs.y),
        }
    }
}

impl<N> Pos2DExt<N>
where
    N: Signed + Copy,
{
    // Returns a position flipped around `center_point`.
    pub fn flip(mut self, center_point: Self) -> Option<Self> {
        let x_diff = self.x - center_point.x;
        let y_diff = self.y - center_point.y;

        self.x = center_point.x - x_diff;
        self.y = center_point.y - y_diff;

        Some(self)
    }

    // Returns a position rotated once around `center_point`.
    fn rot_once(mut self, center_point: Self) -> Option<Self> {
        if self == center_point {
            return Some(self);
        }

        let x_diff = self.x - center_point.x;
        let y_diff = self.y - center_point.y;

        if x_diff.is_zero() {
            self.x = center_point.x + y_diff;
            self.y = center_point.y;
        } else if y_diff.is_zero() {
            self.y = center_point.y - x_diff;
            self.x = center_point.x;
        } else {
            match (x_diff.is_positive(), y_diff.is_positive()) {
                // Top left corner
                (true, true) => self.y = center_point.y - y_diff,
                // Bottom left corner
                (true, false) => self.x = center_point.x - x_diff,
                // Bottom right corner
                (false, true) => self.y = center_point.y + y_diff,
                // Top right corner
                (false, false) => self.x = center_point.x + x_diff,
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
pub struct SurroundingLineIter<N> {
    cur_pos: Pos2DExt<N>,
    change: fn(&Pos2DExt<N>) -> Option<Pos2DExt<N>>,
}

impl<N> SurroundingLineIter<N> {
    pub fn new(pos: Pos2DExt<N>, change: fn(&Pos2DExt<N>) -> Option<Pos2DExt<N>>) -> Self {
        Self {
            cur_pos: pos,
            change,
        }
    }
}

impl<N: Copy> Iterator for SurroundingLineIter<N> {
    type Item = Pos2DExt<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_pos = (self.change)(&self.cur_pos)?;
        Some(self.cur_pos)
    }
}

#[derive(Debug)]
pub struct StepIter<F, N> {
    cur_pos: Pos2DExt<N>,
    change: F,
}

impl<F, N> StepIter<F, N> {
    pub fn new(pos: Pos2DExt<N>, change: F) -> Self {
        Self {
            cur_pos: pos,
            change,
        }
    }
}

impl<F, N> Iterator for StepIter<F, N>
where
    F: FnMut(Pos2DExt<N>) -> Option<Pos2DExt<N>>,
    N: Copy,
{
    type Item = Pos2DExt<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_pos = (self.change)(self.cur_pos)?;
        Some(self.cur_pos)
    }
}

// -------------------------------------------------- //

impl<N> TryFrom<Pos2DExt<N>> for Pos2D
where
    usize: TryFrom<N>,
{
    type Error = <usize as TryFrom<N>>::Error;
    fn try_from(value: Pos2DExt<N>) -> Result<Self, Self::Error> {
        Ok(Pos2D::new(
            usize::try_from(value.x)?,
            usize::try_from(value.y)?,
        ))
    }
}
