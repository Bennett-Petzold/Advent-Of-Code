use std::ops::{Add, Sub};

/// Signed behavior for usize
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedUsize {
    positive: bool,
    value: usize,
}

impl SignedUsize {
    #[inline]
    pub fn flip(self) -> Self {
        Self {
            positive: !self.positive,
            value: self.value,
        }
    }

    pub fn is_nonnegative(&self) -> bool {
        // Yes, this representation does have signed zero
        self.positive || (self.value == 0)
    }

    pub fn is_nonpositive(&self) -> bool {
        // Yes, this representation does have signed zero
        (!self.positive) || (self.value == 0)
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn is_positive(&self) -> bool {
        self.positive && (self.value != 0)
    }

    pub fn is_negative(&self) -> bool {
        (!self.positive) && (self.value != 0)
    }

    /// Positive is true, negative is false.
    ///
    /// Undefined return for zero.
    pub fn sign(&self) -> bool {
        self.positive
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

impl From<usize> for SignedUsize {
    fn from(value: usize) -> Self {
        Self {
            positive: true,
            value,
        }
    }
}

impl Add for SignedUsize {
    type Output = Option<Self>;

    fn add(mut self, rhs: Self) -> Self::Output {
        if self.positive == rhs.positive {
            self.value = self.value.checked_add(rhs.value)?;
        } else if self.value >= rhs.value {
            self.value -= rhs.value;
        } else {
            self.value = rhs.value - self.value;
            self.positive = !self.positive;
        }

        Some(self)
    }
}

impl Add<usize> for SignedUsize {
    type Output = Option<Self>;

    fn add(mut self, rhs: usize) -> Self::Output {
        if self.positive {
            self.value = self.value.checked_add(rhs)?;
        } else if self.value >= rhs {
            self.value -= rhs;
        } else {
            self.value = rhs - self.value;
            self.positive = !self.positive;
        }

        Some(self)
    }
}

impl Add<SignedUsize> for usize {
    type Output = Option<Self>;

    fn add(self, rhs: SignedUsize) -> Self::Output {
        if rhs.positive {
            self.checked_add(rhs.value)
        } else if self >= rhs.value {
            Some(self - rhs.value)
        } else {
            None
        }
    }
}

impl Sub for SignedUsize {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.add(rhs.flip())
    }
}

impl Sub<usize> for SignedUsize {
    type Output = Option<Self>;

    fn sub(mut self, rhs: usize) -> Self::Output {
        if !self.positive {
            self.value = self.value.checked_add(rhs)?;
        } else if self.value >= rhs {
            self.value -= rhs;
        } else {
            self.value = rhs - self.value;
            self.positive = !self.positive;
        }

        Some(self)
    }
}

impl Sub<SignedUsize> for usize {
    type Output = Option<Self>;

    fn sub(self, rhs: SignedUsize) -> Self::Output {
        if !rhs.positive {
            self.checked_add(rhs.value)
        } else if self >= rhs.value {
            Some(self - rhs.value)
        } else {
            None
        }
    }
}

pub struct NegativeSignedUsize;

impl TryInto<usize> for SignedUsize {
    type Error = NegativeSignedUsize;
    fn try_into(self) -> Result<usize, Self::Error> {
        if self.is_nonnegative() {
            Ok(self.value)
        } else {
            Err(NegativeSignedUsize)
        }
    }
}
