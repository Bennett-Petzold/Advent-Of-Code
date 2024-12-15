use std::ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign};

use num::{Num, One, Signed, Unsigned, Zero};

/// SignedWrapper behavior for unsigned types
#[derive(Debug, Clone, Copy)]
pub struct SignedWrapper<N> {
    positive: bool,
    value: N,
}

impl<N> PartialEq for SignedWrapper<N>
where
    N: Zero + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.value.is_zero() && other.value.is_zero() {
            true
        } else {
            (self.positive == other.positive) && self.value.eq(&other.value)
        }
    }
}

impl<N> Eq for SignedWrapper<N> where N: Zero + Eq {}

impl<N> PartialOrd for SignedWrapper<N>
where
    N: Zero + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.value.is_zero() && other.value.is_zero() {
            Some(std::cmp::Ordering::Equal)
        } else {
            Some(
                self.positive
                    .cmp(&other.positive)
                    .then(self.value.partial_cmp(&other.value)?),
            )
        }
    }
}

impl<N> Ord for SignedWrapper<N>
where
    N: Zero + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.value.is_zero() && other.value.is_zero() {
            std::cmp::Ordering::Equal
        } else {
            self.positive
                .cmp(&other.positive)
                .then(self.value.cmp(&other.value))
        }
    }
}

impl<N> Zero for SignedWrapper<N>
where
    N: Zero + PartialOrd + SubAssign + AddAssign + Sub<Output = N>,
{
    fn zero() -> Self {
        Self {
            positive: true,
            value: N::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    fn set_zero(&mut self) {
        self.value.set_zero();
    }
}

impl<N: One + PartialEq> One for SignedWrapper<N> {
    fn one() -> Self {
        Self {
            positive: true,
            value: N::one(),
        }
    }

    fn is_one(&self) -> bool {
        self.value.is_one()
    }

    fn set_one(&mut self) {
        self.positive = true;
        self.value.set_one();
    }
}

impl<N> Neg for SignedWrapper<N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            positive: !self.positive,
            value: self.value,
        }
    }
}

impl<N> Num for SignedWrapper<N>
where
    N: Num + PartialOrd + SubAssign + AddAssign,
{
    type FromStrRadixErr = N::FromStrRadixErr;
    fn from_str_radix(mut s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let neg_sign = s.starts_with('-');
        if neg_sign {
            s = &s['-'.len_utf8()..];
        }

        N::from_str_radix(s, radix).map(|num| Self {
            positive: !neg_sign,
            value: num,
        })
    }
}

impl<N> Signed for SignedWrapper<N>
where
    N: Copy + Zero + Sub<Output = N> + PartialOrd + SubAssign + AddAssign + Num,
{
    fn abs(&self) -> Self {
        Self {
            positive: true,
            value: self.value,
        }
    }

    fn is_positive(&self) -> bool {
        self.positive
    }

    fn is_negative(&self) -> bool {
        !self.positive
    }

    fn signum(&self) -> Self {
        if self.value.is_zero() {
            Self {
                positive: true,
                value: N::zero(),
            }
        } else {
            Self {
                positive: self.positive,
                value: N::one(),
            }
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if *self <= *other {
            Self::zero()
        } else {
            *self - *other
        }
    }
}

impl<N: Unsigned> From<N> for SignedWrapper<N> {
    fn from(value: N) -> Self {
        Self {
            positive: true,
            value,
        }
    }
}

impl<N> Add for SignedWrapper<N>
where
    N: Sub<Output = N> + AddAssign + SubAssign + PartialOrd,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        if self.positive == rhs.positive {
            self.value += rhs.value;
        } else if self.value >= rhs.value {
            self.value -= rhs.value;
        } else {
            self.value = rhs.value - self.value;
            self.positive = !self.positive;
        }

        self
    }
}

impl<N> Add<N> for SignedWrapper<N>
where
    N: AddAssign + SubAssign + Sub<Output = N> + PartialOrd,
{
    type Output = Self;

    fn add(mut self, rhs: N) -> Self::Output {
        if self.positive {
            self.value += rhs;
        } else if self.value >= rhs {
            self.value -= rhs;
        } else {
            self.value = rhs - self.value;
            self.positive = !self.positive;
        }

        self
    }
}

impl<N> Sub for SignedWrapper<N>
where
    N: Sub<Output = N> + AddAssign + SubAssign + PartialOrd,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.add(-rhs)
    }
}

impl<N> Sub<N> for SignedWrapper<N>
where
    N: AddAssign + SubAssign + Sub<Output = N> + PartialOrd,
{
    type Output = Self;

    fn sub(mut self, rhs: N) -> Self::Output {
        if !self.positive {
            self.value = rhs;
        } else if self.value >= rhs {
            self.value -= rhs;
        } else {
            self.value = rhs - self.value;
            self.positive = !self.positive;
        }

        self
    }
}

impl<N: Mul<Output = N>> Mul for SignedWrapper<N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            // XNOR
            positive: !(self.positive ^ rhs.positive),
            value: self.value * rhs.value,
        }
    }
}

impl<N: Mul<Output = N>> Mul<N> for SignedWrapper<N> {
    type Output = Self;

    fn mul(self, rhs: N) -> Self::Output {
        Self {
            positive: self.positive,
            value: self.value * rhs,
        }
    }
}

impl<N: Div<Output = N>> Div for SignedWrapper<N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            // XNOR
            positive: !(self.positive ^ rhs.positive),
            value: self.value / rhs.value,
        }
    }
}

impl<N: Div<Output = N>> Div<N> for SignedWrapper<N> {
    type Output = Self;

    fn div(self, rhs: N) -> Self::Output {
        Self {
            positive: self.positive,
            value: self.value / rhs,
        }
    }
}

impl<N: Rem<Output = N>> Rem for SignedWrapper<N> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            // Remainder is always positive
            positive: true,
            value: self.value % rhs.value,
        }
    }
}

impl<N: Rem<Output = N>> Rem<N> for SignedWrapper<N> {
    type Output = Self;

    fn rem(self, rhs: N) -> Self::Output {
        Self {
            // Remainder is always positive
            positive: true,
            value: self.value % rhs,
        }
    }
}

pub struct NegativeSignedWrapper;

impl<N> SignedWrapper<N>
where
    N: Copy + Zero + Div<Output = N> + Sub<Output = N> + PartialOrd + SubAssign + AddAssign + Num,
{
    /// Converts this into the unsigned form, if nonnegative.
    pub fn unsigned(self) -> Result<N, NegativeSignedWrapper> {
        if self.is_positive() || self.is_zero() {
            Ok(self.value)
        } else {
            Err(NegativeSignedWrapper)
        }
    }
}
