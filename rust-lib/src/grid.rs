use std::{
    cmp::Ordering,
    fmt::Display,
    io::Write,
    ops::{Add, Sub},
};

use thiserror::Error;

use crate::{
    iter::{ArrayIter, ToExactIter},
    signed::SignedUsize,
};

/// Position in a 2D grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos2D {
    pub x: usize,
    pub y: usize,
}

impl Display for Pos2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Pos2D {
    pub const fn new(x: usize, y: usize) -> Self {
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

    pub fn surrounding_pos(&self) -> impl DoubleEndedIterator<Item = Self> {
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
        F: FnMut(Pos2D) -> Option<Pos2D>,
    {
        StepIter::new(*self, step)
    }

    pub fn get_arr_char<S, A>(&self, arr: A) -> Option<char>
    where
        S: AsRef<str>,
        A: AsRef<[S]>,
    {
        arr.as_ref().get(self.y)?.as_ref().chars().nth(self.x)
    }

    // All edges, ordered by highest and then by leftmost
    //
    // e.g. (0 0), (0 1), (1 0), (1 1)
    pub fn order_top_left(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.y.cmp(&rhs.y).then(lhs.x.cmp(&rhs.x))
    }

    // All edges, ordered by leftmost and then by highest
    //
    // e.g. (0 0), (1 0), (0 1), (1 1)
    pub fn order_left_top(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.x.cmp(&rhs.x).then(lhs.y.cmp(&rhs.y))
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

#[derive(Debug)]
pub struct StepIter<F> {
    cur_pos: Pos2D,
    change: F,
}

impl<F> StepIter<F> {
    pub fn new(pos: Pos2D, change: F) -> Self {
        Self {
            cur_pos: pos,
            change,
        }
    }
}

impl<F> Iterator for StepIter<F>
where
    F: FnMut(Pos2D) -> Option<Pos2D>,
{
    type Item = Pos2D;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_pos = (self.change)(self.cur_pos)?;
        Some(self.cur_pos)
    }
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
/// Rectangle grid with a flat inner representation.
pub struct RectangleGrid<T> {
    inner: Box<[T]>,
    x_max: usize,
    y_max: usize,
}

#[derive(Debug, Error)]
#[error("At least one line of the iterator was a different length.")]
pub struct NonRectangleInput;

impl<T> RectangleGrid<T> {
    /// Attempt to construct this from a 2D iterator.
    ///
    /// May fail when the iterator is non-square, but is not guaranteed to.
    pub fn try_from_iter<I, C>(iter: I) -> Result<Self, NonRectangleInput>
    where
        I: IntoIterator<Item = C>,
        C: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();

        let mut y_max = 0;
        let mut x_max = 0;
        let mut inner = Vec::new();

        if let Some(first) = iter.next() {
            inner = first.into_iter().collect();
            x_max = inner.len();

            inner.extend(iter.flat_map(|x| x.into_iter()));

            if (inner.len() % x_max) != 0 {
                return Err(NonRectangleInput);
            }

            y_max = inner.len() / x_max;
        }

        Ok(Self {
            inner: inner.into_boxed_slice(),
            x_max,
            y_max,
        })
    }

    /// Attempt to construct this from a 2D iterator.
    ///
    /// Will fail when iterator is non-square.
    pub fn try_from_iter_strict<I, C>(iter: I) -> Result<Self, NonRectangleInput>
    where
        I: IntoIterator<Item = C>,
        C: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();

        let mut y_max = 0;
        let mut x_max = 0;
        let mut inner = Vec::new();

        if let Some(first) = iter.next() {
            inner = first.into_iter().collect();
            x_max = inner.len();

            for x in iter {
                let mut x: Vec<_> = x.into_iter().collect();
                if x.len() != x_max {
                    return Err(NonRectangleInput);
                }
                inner.append(&mut x);
            }

            y_max = inner.len() / x_max;
        }

        Ok(Self {
            inner: inner.into_boxed_slice(),
            x_max,
            y_max,
        })
    }
}

impl<T> RectangleGrid<T> {
    pub fn x_max(&self) -> usize {
        self.x_max
    }

    pub fn y_max(&self) -> usize {
        self.y_max
    }

    /// Returns true if `pos` is within this grid's dimensions.
    pub fn in_grid(&self, pos: Pos2D) -> bool {
        (pos.y < self.y_max) && (pos.x < self.x_max)
    }

    /// Flatten the pose to an internal index.
    ///
    /// Invalid if pos in not in this map.
    fn flat_pos(&self, pos: Pos2D) -> usize {
        (pos.y * self.x_max) + pos.x
    }

    pub fn get(&self, pos: Pos2D) -> Option<&T> {
        self.in_grid(pos).then(|| &self.inner[self.flat_pos(pos)])
    }

    pub fn get_mut(&mut self, pos: Pos2D) -> Option<&mut T> {
        self.in_grid(pos)
            .then(|| &mut self.inner[self.flat_pos(pos)])
    }

    pub fn lines(&self) -> impl ArrayIter<&[T]> {
        self.inner.chunks(self.x_max)
    }

    pub fn lines_mut(&mut self) -> impl ArrayIter<&mut [T]> {
        self.inner.chunks_mut(self.x_max)
    }

    pub fn items(&self) -> impl ArrayIter<&T> {
        self.inner.iter()
    }

    pub fn items_mut(&mut self) -> impl ArrayIter<&mut T> {
        self.inner.iter_mut()
    }

    /// All positions, starting at top left and moving right before down.
    ///
    /// Output order in 1x1: (0 0), (0 1), (1 0), (1 1)
    pub fn positions(&self) -> impl ArrayIter<Pos2D> + use<'_, T> {
        ToExactIter::new(
            (0..self.y_max).flat_map(|y| (0..self.x_max).map(move |x| Pos2D::new(x, y))),
            self.y_max * self.x_max,
        )
    }
}

impl<T: Copy> RectangleGrid<T> {
    pub fn at(&self, pos: Pos2D) -> Option<T> {
        self.in_grid(pos).then_some(self.inner[self.flat_pos(pos)])
    }
}

impl<T> IntoIterator for RectangleGrid<T> {
    type Item = T;
    type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Box<[T]> as IntoIterator>::into_iter(self.inner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Immutable entry in the grid, keeping position data.
pub struct GridEntry<'a, T> {
    position: Pos2D,
    pub value: &'a T,
}

impl<T> GridEntry<'_, T> {
    pub fn position(&self) -> Pos2D {
        self.position
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Mutable entry in the grid, keeping position data.
pub struct GridEntryMut<'a, T> {
    position: Pos2D,
    pub value: &'a mut T,
}

impl<T> GridEntryMut<'_, T> {
    pub fn position(&self) -> Pos2D {
        self.position
    }
}

impl<T> RectangleGrid<T> {
    /// Produces each item with its position.
    pub fn positioned_items(&self) -> impl ArrayIter<GridEntry<'_, T>> {
        self.positions()
            .zip(self.items())
            .map(|(position, value)| GridEntry { position, value })
    }

    /// Produces each mutable item with its position.
    pub fn positioned_items_mut(&mut self) -> impl ExactSizeIterator<Item = GridEntryMut<'_, T>> {
        ToExactIter::new(
            (0..self.y_max)
                .flat_map(|y| (0..self.x_max).map(move |x| Pos2D::new(x, y)))
                .zip(self.inner.iter_mut())
                .map(|(position, value)| GridEntryMut { position, value }),
            self.y_max * self.x_max,
        )
    }
}

impl<T: Clone> RectangleGrid<T> {
    // Evenly the outside with a given value
    pub fn pad_surrounding(&self, value: T) -> Self {
        let horizontal_pad = std::iter::repeat_n(value.clone(), self.x_max + 1);
        let vertical_pad = self.lines().flat_map(|line| {
            std::iter::once(value.clone())
                .chain(line.iter().cloned())
                .chain(std::iter::once(value.clone()))
        });

        let new_arr = horizontal_pad
            .clone()
            .chain(vertical_pad)
            .chain(horizontal_pad)
            .collect();
        Self {
            inner: new_arr,
            x_max: self.x_max + 1,
            y_max: self.y_max + 1,
        }
    }
}

impl<T> RectangleGrid<T> {
    /// Formats the grid's items with some function `to_str` into `sink`.
    pub fn print<W, F>(&self, sink: &mut W, mut to_str: F) -> Result<(), std::io::Error>
    where
        W: Write,
        F: FnMut(&T) -> &str,
    {
        for line in self.lines() {
            for item in line {
                write!(sink, "{}", (to_str)(item))?;
            }
            writeln!(sink)?;
        }

        Ok(())
    }
}
