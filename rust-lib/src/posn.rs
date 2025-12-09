use std::{
    array,
    cmp::Ordering,
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use num::{cast::AsPrimitive, CheckedSub};

/// Position in a N-dimensional grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos<T, const N: usize> {
    pub coordinates: [T; N],
}

impl<T: PartialOrd, const N: usize> PartialOrd for Pos<T, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.coordinates
            .iter()
            .zip(&other.coordinates)
            .map(|(lhs, rhs)| (*lhs).partial_cmp(rhs))
            .reduce(|acc, op| {
                let acc = acc?;
                let op = op?;
                Some(acc.then(op))
            })
            // N = 0 is equal.
            .unwrap_or(Some(Ordering::Equal))
    }
}

impl<T: Ord, const N: usize> Ord for Pos<T, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coordinates
            .iter()
            .zip(&other.coordinates)
            .map(|(lhs, rhs)| (*lhs).cmp(rhs))
            .reduce(|acc, op| acc.then(op))
            // N = 0 is equal.
            .unwrap_or(Ordering::Equal)
    }
}

impl<T: Debug, const N: usize> Display for Pos<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.coordinates)
    }
}

impl<T, const N: usize> Pos<T, N> {
    pub const fn new(coordinates: [T; N]) -> Self {
        Self { coordinates }
    }
}

impl<T, const N: usize> From<[T; N]> for Pos<T, N> {
    fn from(coordinates: [T; N]) -> Self {
        Self { coordinates }
    }
}

impl<T: AddAssign + Add<Output = T>, const N: usize> Add for Pos<T, N> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.coordinates
            .iter_mut()
            .zip(rhs.coordinates)
            .for_each(|(rhs, lhs)| *rhs += lhs);
        self
    }
}

impl<T, const N: usize> Pos<T, N>
where
    T: Sum + AsPrimitive<f32>,
    for<'a> &'a T: Mul<Output = T>,
{
    pub fn euclid_dist_from_origin(&self) -> f32 {
        let squares: T = self.coordinates.iter().map(|dim| dim * dim).sum();
        (squares.as_()).sqrt()
    }
}

impl<T, const N: usize> Pos<T, N>
where
    T: CheckedSub,
    for<'a> &'a T: Sub<Output = T>,
{
    pub fn abs_diff(&self, other: &Self) -> Self {
        let mut values = self
            .coordinates
            .iter()
            .zip(other.coordinates.iter())
            .map(|(lhs, rhs)| (*lhs).checked_sub(rhs).unwrap_or_else(|| rhs - lhs));
        Self {
            coordinates: array::from_fn(|_| values.next().expect("Dimensions match")),
        }
    }
}

impl<T, const N: usize> Pos<T, N>
where
    T: Sum + CheckedSub + AsPrimitive<f32>,
    T: CheckedSub,
    for<'a> &'a T: Sub<Output = T> + Mul<Output = T>,
{
    pub fn euclid_dist(&self, other: &Self) -> f32 {
        // Euclid distance from each other is the same as the absolute
        // difference's euclid distance from origin.
        self.abs_diff(other).euclid_dist_from_origin()
    }
}
