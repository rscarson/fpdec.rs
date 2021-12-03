// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{Decimal, MAX_N_FRAC_DIGITS};

/// Checked multiplication.
/// Computes `self * rhs`.
/// Returns `None` if the result can not be represented by the `Output` type.
pub trait CheckedMul<Rhs = Self> {
    /// The resulting type after applying `checked_mul`.
    type Output;
    /// Returns `Some(self * rhs)` or `None` if the result can not be
    /// represented by the `Output` type.
    fn checked_mul(self, rhs: Rhs) -> Self::Output;
}

impl CheckedMul<Decimal> for Decimal {
    type Output = Option<Self>;

    #[inline]
    fn checked_mul(self, rhs: Decimal) -> Self::Output {
        let n_frac_digits = self.n_frac_digits + rhs.n_frac_digits;
        if n_frac_digits > MAX_N_FRAC_DIGITS {
            // TODO: try normalized operands
            return None;
        }
        Some(Self {
            coeff: i128::checked_mul(self.coeff, rhs.coeff)?,
            n_frac_digits,
        })
    }
}

forward_ref_binop!(impl CheckedMul, checked_mul);

#[cfg(test)]
mod checked_mul_decimal_tests {
    use super::*;

    #[test]
    fn test_checked_mul() {
        let x = Decimal::new_raw(1234567890, 4);
        let y = x.checked_mul(x).unwrap();
        assert_eq!(y.coeff, x.coeff * x.coeff);
        assert_eq!(y.n_frac_digits, 2 * x.n_frac_digits);
        let z = x.checked_mul(Decimal::NEG_ONE).unwrap();
        assert_eq!(z.coeff, -x.coeff);
        assert_eq!(z.n_frac_digits, x.n_frac_digits);
        let x = Decimal::new_raw(1234567890, 5);
        let y = Decimal::new_raw(890, 1);
        let z = x.checked_mul(y).unwrap();
        assert_eq!(z.coeff, x.coeff * y.coeff);
        assert_eq!(z.n_frac_digits, x.n_frac_digits + y.n_frac_digits);
        let z = y.checked_mul(x).unwrap();
        assert_eq!(z.coeff, x.coeff * y.coeff);
        assert_eq!(z.n_frac_digits, x.n_frac_digits + y.n_frac_digits);
        let y = Decimal::new_raw(-1, 3);
        let z = x.checked_mul(y).unwrap();
        assert_eq!(z.coeff, -x.coeff);
        assert_eq!(z.n_frac_digits, x.n_frac_digits + y.n_frac_digits);
    }

    #[test]
    fn test_checked_mul_pos_overflow() {
        let x = Decimal::new_raw(i128::MAX / 2 + 1, 4);
        let y = x.checked_mul(Decimal::TWO);
        assert!(y.is_none());
    }

    #[test]
    fn test_checked_mul_neg_overflow() {
        let x = Decimal::new_raw(i128::MIN, 2);
        let y = x.checked_mul(Decimal::NEG_ONE);
        assert!(y.is_none());
    }

    #[test]
    fn test_checked_mul_ref() {
        let x = Decimal::new_raw(12345, 3);
        let y = Decimal::new_raw(12345, 1);
        let z = x.checked_mul(y).unwrap();
        assert_eq!(z.coeff, (&x).checked_mul(y).unwrap().coeff);
        assert_eq!(z.coeff, x.checked_mul(&y).unwrap().coeff);
        assert_eq!(z.coeff, (&x).checked_mul(&y).unwrap().coeff);
    }
}

macro_rules! impl_checked_mul_decimal_and_int {
    () => {
        impl_checked_mul_decimal_and_int!(
            u8, i8, u16, i16, u32, i32, u64, i64, i128
        );
    };
    ($($t:ty),*) => {
        $(
        impl CheckedMul<$t> for Decimal {
            type Output = Option<Decimal>;

            #[inline]
            fn checked_mul(self, rhs: $t) -> Self::Output {
                Some(Self {
                    coeff: i128::checked_mul(self.coeff, rhs as i128)?,
                    n_frac_digits: self.n_frac_digits,
                })
            }
        }

        impl CheckedMul<Decimal> for $t {
            type Output = Option<Decimal>;

            #[inline]
            fn checked_mul(self, rhs: Decimal) -> Self::Output {
                Some(Decimal {
                    coeff: i128::checked_mul(self as i128, rhs.coeff)?,
                    n_frac_digits: rhs.n_frac_digits,
                })
            }
        }
        )*
    }
}

impl_checked_mul_decimal_and_int!();
forward_ref_binop_decimal_int!(impl CheckedMul, checked_mul);

#[cfg(test)]
#[allow(clippy::neg_multiply)]
mod checked_mul_integer_tests {
    use super::*;

    macro_rules! gen_checked_mul_integer_tests {
        ($func:ident, $t:ty, $p:expr, $coeff:expr) => {
            #[test]
            fn $func() {
                let d = Decimal::new_raw($coeff, $p);
                let i = <$t>::MAX;
                let r = d.checked_mul(i).unwrap();
                assert_eq!(r.coeff, i as i128 * $coeff);
                assert_eq!(r.coeff, (&d).checked_mul(i).unwrap().coeff);
                assert_eq!(r.coeff, d.checked_mul(&i).unwrap().coeff);
                assert_eq!(r.coeff, (&d).checked_mul(&i).unwrap().coeff);
                let z = CheckedMul::checked_mul(i, d).unwrap();
                assert_eq!(z.coeff, r.coeff);
                assert_eq!(
                    z.coeff,
                    CheckedMul::checked_mul(&i, d).unwrap().coeff
                );
                assert_eq!(
                    z.coeff,
                    CheckedMul::checked_mul(i, &d).unwrap().coeff
                );
                assert_eq!(
                    z.coeff,
                    CheckedMul::checked_mul(&i, &d).unwrap().coeff
                );
                let d = Decimal::new_raw(i128::MAX, $p);
                let i: $t = 2;
                let z = d.checked_mul(i);
                assert!(z.is_none());
            }
        };
    }

    gen_checked_mul_integer_tests!(test_checked_mul_u8, u8, 2, -1);
    gen_checked_mul_integer_tests!(test_checked_mul_i8, i8, 0, 123);
    gen_checked_mul_integer_tests!(test_checked_mul_u16, u16, 4, 11);
    gen_checked_mul_integer_tests!(test_checked_mul_i16, i16, 4, 1234567);
    gen_checked_mul_integer_tests!(test_checked_mul_u32, u32, 1, 0);
    gen_checked_mul_integer_tests!(test_checked_mul_i32, i32, 9, -1234);
    gen_checked_mul_integer_tests!(test_checked_mul_u64, u64, 3, 321);
    gen_checked_mul_integer_tests!(
        test_checked_mul_i64,
        i64,
        7,
        -12345678901234567890
    );

    #[test]
    fn test_checked_mul_i128() {
        let coeff = 748_i128;
        let d = Decimal::new_raw(coeff, 2);
        let i = 12345_i128;
        let r = d.checked_mul(i).unwrap();
        assert_eq!(r.coeff, i as i128 * coeff);
        assert_eq!(r.coeff, (&d).checked_mul(i).unwrap().coeff);
        assert_eq!(r.coeff, d.checked_mul(&i).unwrap().coeff);
        assert_eq!(r.coeff, (&d).checked_mul(&i).unwrap().coeff);
        let z = CheckedMul::checked_mul(i, d).unwrap();
        assert_eq!(z.coeff, r.coeff);
        assert_eq!(z.coeff, CheckedMul::checked_mul(&i, d).unwrap().coeff);
        assert_eq!(z.coeff, CheckedMul::checked_mul(i, &d).unwrap().coeff);
        assert_eq!(z.coeff, CheckedMul::checked_mul(&i, &d).unwrap().coeff);
        let i = u64::MAX as i128;
        let d = Decimal::new_raw(i, 3);
        let z = d.checked_mul(i);
        assert!(z.is_none());
    }
}