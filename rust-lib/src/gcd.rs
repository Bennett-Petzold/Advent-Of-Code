use num::{integer::ExtendedGcd, Integer, Signed};

/// Returns the gcd coefficients of `lhs`, `rhs` if they are coprime.
///
/// Return: Option<(lhs coefficient, rhs coefficient)>
pub fn coprime_coefficients<N: std::fmt::Display + Integer + Clone + Signed>(
    lhs: N,
    rhs: N,
) -> Option<(N, N)> {
    let ExtendedGcd { gcd, x, y } = lhs.extended_gcd(&rhs);
    if gcd == N::one() {
        Some((x, y))
    } else {
        None
    }
}
