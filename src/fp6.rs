//! This module implements arithmetic over the extension field Fp6,
//! defined with irreducible polynomial v^3 - v - 2.

use core::convert::TryFrom;
use core::fmt;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_core::{CryptoRng, RngCore};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::fp::Fp;
use crate::fp2::Fp2;

const MODULUS_MINUS_ONE_DIV_TWO: [u64; 6] = [
    0xbfff598000000000,
    0x953f2fe05a3de000,
    0xc22d66eed23dc5ea,
    0xb9538f1d34e3dc07,
    0xa978d997e2ea6efc,
    0x0007ffd6605a3d77,
];

const MODULUS_PLUS_ONE_DIV_TWO: [u64; 6] = [
    0xbfff598000000001,
    0x953f2fe05a3de000,
    0xc22d66eed23dc5ea,
    0xb9538f1d34e3dc07,
    0xa978d997e2ea6efc,
    0x0007ffd6605a3d77,
];

#[derive(Copy, Clone)]
/// An element of the extension GF(p^6)
pub struct Fp6 {
    /// First coefficient
    pub c0: Fp2,
    /// Second coefficient
    pub c1: Fp2,
    /// Third coefficient
    pub c2: Fp2,
}

impl fmt::Debug for Fp6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} + ({:?})*v + ({:?})*v^2", self.c0, self.c1, self.c2)
    }
}

impl Default for Fp6 {
    fn default() -> Self {
        Fp6::zero()
    }
}

impl zeroize::DefaultIsZeroes for Fp6 {}

impl From<Fp2> for Fp6 {
    fn from(f: Fp2) -> Self {
        Fp6 {
            c0: f,
            c1: Fp2::zero(),
            c2: Fp2::zero(),
        }
    }
}

impl From<Fp> for Fp6 {
    fn from(f: Fp) -> Self {
        Fp6::from(Fp2::from(f))
    }
}

impl From<u64> for Fp6 {
    /// Converts a 64-bit value into a filed element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently preformed.
    fn from(value: u64) -> Self {
        Fp6::from(Fp::new(value))
    }
}

impl From<u32> for Fp6 {
    /// Converts a 32-bit value into a filed element.
    fn from(value: u32) -> Self {
        Fp6::from(Fp::new(value as u64))
    }
}

impl From<u16> for Fp6 {
    /// Converts a 16-bit value into a filed element.
    fn from(value: u16) -> Self {
        Fp6::from(Fp::new(value as u64))
    }
}

impl From<u8> for Fp6 {
    /// Converts an 8-bit value into a filed element.
    fn from(value: u8) -> Self {
        Fp6::from(Fp::new(value as u64))
    }
}

impl ConstantTimeEq for Fp6 {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.c0.ct_eq(&other.c0) & self.c1.ct_eq(&other.c1) & self.c2.ct_eq(&other.c2)
    }
}

impl Eq for Fp6 {}
impl PartialEq for Fp6 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl ConditionallySelectable for Fp6 {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Fp6 {
            c0: Fp2::conditional_select(&a.c0, &b.c0, choice),
            c1: Fp2::conditional_select(&a.c1, &b.c1, choice),
            c2: Fp2::conditional_select(&a.c2, &b.c2, choice),
        }
    }
}

impl<'a> Neg for &'a Fp6 {
    type Output = Fp6;

    #[inline]
    fn neg(self) -> Fp6 {
        self.neg()
    }
}

impl Neg for Fp6 {
    type Output = Fp6;

    #[inline]
    fn neg(self) -> Fp6 {
        -&self
    }
}

impl<'a, 'b> Sub<&'b Fp6> for &'a Fp6 {
    type Output = Fp6;

    #[inline]
    fn sub(self, rhs: &'b Fp6) -> Fp6 {
        self.sub(rhs)
    }
}

impl<'a, 'b> Add<&'b Fp6> for &'a Fp6 {
    type Output = Fp6;

    #[inline]
    fn add(self, rhs: &'b Fp6) -> Fp6 {
        self.add(rhs)
    }
}

impl<'a, 'b> Mul<&'b Fp6> for &'a Fp6 {
    type Output = Fp6;

    #[inline]
    fn mul(self, rhs: &'b Fp6) -> Fp6 {
        self.mul(rhs)
    }
}

impl_binops_additive!(Fp6, Fp6);
impl_binops_multiplicative!(Fp6, Fp6);

impl Fp6 {
    /// Creates a new field element from a [u64; 6] value.
    /// The value is converted to Montgomery form by computing
    /// (a.R^0 * R^2) / R = a.R
    pub const fn new(value: [u64; 6]) -> Self {
        Fp6 {
            c0: Fp2 {
                c0: Fp::new(value[0]),
                c1: Fp::new(value[1]),
            },
            c1: Fp2 {
                c0: Fp::new(value[2]),
                c1: Fp::new(value[3]),
            },
            c2: Fp2 {
                c0: Fp::new(value[4]),
                c1: Fp::new(value[5]),
            },
        }
    }

    #[inline]
    /// The additive identity
    pub const fn zero() -> Self {
        Fp6 {
            c0: Fp2::zero(),
            c1: Fp2::zero(),
            c2: Fp2::zero(),
        }
    }

    #[inline]
    /// The multiplicative identity
    pub const fn one() -> Self {
        Fp6 {
            c0: Fp2::one(),
            c1: Fp2::zero(),
            c2: Fp2::zero(),
        }
    }

    /// Checks whether this element is zero or not
    pub fn is_zero(&self) -> Choice {
        self.c0.is_zero() & self.c1.is_zero() & self.c2.is_zero()
    }

    /// Generates a random element
    pub fn random(mut rng: impl CryptoRng + RngCore) -> Self {
        Fp6 {
            c0: Fp2::random(&mut rng),
            c1: Fp2::random(&mut rng),
            c2: Fp2::random(&mut rng),
        }
    }

    /// Returns whether or not this element is strictly lexicographically
    /// larger than its negation.
    #[inline]
    pub fn lexicographically_largest(&self) -> Choice {
        // If this element's c1 coefficient is lexicographically largest
        // then it is lexicographically largest. Otherwise, in the event
        // the c1 coefficient is zero and the c0 coefficient is
        // lexicographically largest, then this element is lexicographically
        // largest.

        self.c2.lexicographically_largest()
            | (self.c2.is_zero() & self.c1.lexicographically_largest())
            | (self.c1.is_zero() & self.c0.lexicographically_largest())
    }

    #[inline]
    /// Computes the multiplication of two Fp6 elements
    pub const fn mul(&self, other: &Fp6) -> Fp6 {
        let aa = (&self.c0).mul(&other.c0);
        let bb = (&self.c1).mul(&other.c1);
        let cc = (&self.c2).mul(&other.c2);

        let tmp0 = (&self.c1).add(&self.c2);
        let tmp0b = (&other.c1).add(&other.c2);
        let tmp0 = (&tmp0).mul(&tmp0b);

        let tmp1 = (&self.c0).add(&self.c1);
        let tmp1b = (&other.c0).add(&other.c1);
        let tmp1 = (&tmp1).mul(&tmp1b);

        let tmp2 = (&self.c0).add(&self.c2);
        let tmp2b = (&other.c0).add(&other.c2);
        let tmp2 = (&tmp2).mul(&tmp2b);

        let c0 = (&tmp0).sub(&bb);
        let c0 = (&c0).sub(&cc);
        let c0 = (&c0).double();
        let c0 = (&c0).add(&aa);

        let c1 = (&tmp0).add(&tmp1);
        let c1 = (&c1).sub(&aa);
        let c1 = (&c1).sub(&bb);
        let c1 = (&c1).sub(&bb);
        let c1 = (&c1).add(&cc);

        let c2 = (&tmp2).sub(&aa);
        let c2 = (&c2).add(&bb);

        Fp6 { c0, c1, c2 }
    }

    /// Square this element
    #[inline]
    pub const fn square(&self) -> Self {
        let aa = (&self.c0).mul(&self.c0);
        let bb = (&self.c1).mul(&self.c1);
        let cc = (&self.c2).mul(&self.c2);

        let tmp0 = (&self.c1).add(&self.c2);
        let tmp0 = (&tmp0).mul(&tmp0);

        let tmp1 = (&self.c0).add(&self.c1);
        let tmp1 = (&tmp1).mul(&tmp1);

        let tmp2 = (&self.c0).add(&self.c2);
        let tmp2 = (&tmp2).mul(&tmp2);

        let c0 = (&tmp0).sub(&bb);
        let c0 = (&c0).sub(&cc);
        let c0 = (&c0).double();
        let c0 = (&c0).add(&aa);

        let c1 = (&tmp0).add(&tmp1);
        let c1 = (&c1).sub(&aa);
        let c1 = (&c1).sub(&bb);
        let c1 = (&c1).sub(&bb);
        let c1 = (&c1).add(&cc);

        let c2 = (&tmp2).sub(&aa);
        let c2 = (&c2).add(&bb);

        Fp6 { c0, c1, c2 }
    }

    /// Computes the square root of this element, if it exists.
    /// This operation is not constant time, as it requires going
    /// through a non-constant number of field elements until we reach
    /// a non-square.
    // TODO: Should rng be passed as argument?
    pub fn sqrt_vartime(&self) -> Option<Self> {
        // Cipolla's algorithm
        // https://en.wikipedia.org/wiki/Cipolla%27s_algorithm

        fn mul_in_fp12(a: &[Fp6; 2], b: &[Fp6; 2], beta: &Fp6) -> [Fp6; 2] {
            let v0 = (&a[0]).mul(&b[0]);
            let v1 = (&a[1]).mul(&b[1]);
            let c0 = (&v0).add(&(beta.mul(&v1)));
            let c1 = (&a[0]).add(&a[1]);
            let c1 = (&c1).mul(&b[0].add(&b[1]));
            let c1 = (&c1).sub(&v0);
            let c1 = (&c1).sub(&v1);

            [c0, c1]
        }

        fn exp_vartime_in_fp12(a: &[Fp6; 2], by: &[u64; 6], beta: &Fp6) -> [Fp6; 2] {
            let mut res = [Fp6::one(), Fp6::zero()];
            for e in by.iter().rev() {
                for i in (0..64).rev() {
                    res = mul_in_fp12(&res, &res, beta);

                    if ((*e >> i) & 1) == 1 {
                        res = mul_in_fp12(&res, a, beta);
                    }
                }
            }
            res
        }

        // check = self^((p^6 - 1) // 2)
        //       = self^0x7ffd6605a3d77a978d997e2ea6efcb9538f1d34e3dc07c22d66eed23dc5ea953f2fe05a3de000bfff598000000000
        let check = self.exp_vartime(&MODULUS_MINUS_ONE_DIV_TWO);

        if check != Fp6::one() {
            return None;
        }

        let mut a = Fp6::from(Fp2::one().double());
        while (a.square() - self).exp_vartime(&MODULUS_MINUS_ONE_DIV_TWO) != -Fp6::one() {
            a += Fp6::one();
        }

        let beta = a.square() - self;

        let res = exp_vartime_in_fp12(&[a, Fp6::one()], &MODULUS_PLUS_ONE_DIV_TWO, &beta);
        if res[1] != Fp6::zero() {
            return None;
        }

        if bool::from(self.ct_eq(&res[0].square())) {
            Some(res[0])
        } else {
            None
        }
    }

    /// Add two elements together
    #[inline]
    pub const fn add(&self, rhs: &Self) -> Self {
        Fp6 {
            c0: (&self.c0).add(&rhs.c0),
            c1: (&self.c1).add(&rhs.c1),
            c2: (&self.c2).add(&rhs.c2),
        }
    }

    /// Substract two elements together
    #[inline]
    pub const fn sub(&self, rhs: &Self) -> Self {
        Fp6 {
            c0: (&self.c0).sub(&rhs.c0),
            c1: (&self.c1).sub(&rhs.c1),
            c2: (&self.c2).sub(&rhs.c2),
        }
    }

    /// Negate `&self`
    #[inline]
    pub const fn neg(&self) -> Self {
        Fp6 {
            c0: (&self.c0).neg(),
            c1: (&self.c1).neg(),
            c2: (&self.c2).neg(),
        }
    }

    /// Computes the multiplicative inverse of this field
    /// element, returning None in the case that this element
    /// is zero.
    pub fn invert(&self) -> CtOption<Self> {
        let three = Fp2::new(3);
        let two = Fp2::new(2);
        let four = Fp2::new(4);

        let c0_sq = self.c0.square();
        let c1_sq = self.c1.square();
        let c2_sq = self.c2.square();

        let two_c1 = two * self.c1;
        let c0_c1 = self.c0 * self.c1;

        let inv = self.c0 * c0_sq - self.c0 * self.c1.square()
            + two_c1 * c1_sq
            + (self.c0 - two_c1) * self.c2.square()
            + four * self.c2 * c2_sq
            + two * (self.c0.square() - three * c0_c1) * self.c2;

        let c0 = self.c0.square() - self.c1.square()
            + two * (self.c0 - self.c1) * self.c2
            + self.c2.square();
        let c1 = -(c0_c1 - two * self.c2.square());
        let c2 = self.c1.square() - self.c0 * self.c2 - self.c2.square();

        inv.invert().map(|t| Fp6 {
            c0: c0 * t,
            c1: c1 * t,
            c2: c2 * t,
        })
    }

    /// Exponentiates `self` by `power`, where `power` is a
    /// little-endian order integer exponent.
    pub fn exp(self, by: &[u64; 6]) -> Self {
        let mut res = Self::one();
        for e in by.iter().rev() {
            for i in (0..64).rev() {
                res = res.square();
                let mut tmp = res;
                tmp *= self;
                res.conditional_assign(&tmp, (((e >> i) & 1) as u8).into());
            }
        }
        res
    }

    /// Although this is labeled "vartime", it is only
    /// variable time with respect to the exponent.
    pub fn exp_vartime(&self, by: &[u64; 6]) -> Self {
        let mut res = Self::one();
        for e in by.iter().rev() {
            for i in (0..64).rev() {
                res = res.square();

                if ((*e >> i) & 1) == 1 {
                    res *= self;
                }
            }
        }
        res
    }

    /// Outputs the internal representation as 4 64-bit limbs after Montgomery reduction
    pub const fn to_repr(&self) -> [u64; 6] {
        [
            Fp::montgomery_reduce(self.c0.c0.0, 0).0,
            Fp::montgomery_reduce(self.c0.c1.0, 0).0,
            Fp::montgomery_reduce(self.c1.c0.0, 0).0,
            Fp::montgomery_reduce(self.c1.c1.0, 0).0,
            Fp::montgomery_reduce(self.c2.c0.0, 0).0,
            Fp::montgomery_reduce(self.c2.c1.0, 0).0,
        ]
    }

    /// Converts a `Fp` element into a byte representation in
    /// little-endian byte order.
    pub fn to_bytes(&self) -> [u8; 48] {
        // Turn into canonical form by computing
        // (a.R) / R = a
        let tmp = self.to_repr();

        let mut res = [0u8; 48];
        res[0..8].copy_from_slice(&tmp[0].to_le_bytes());
        res[8..16].copy_from_slice(&tmp[1].to_le_bytes());
        res[16..24].copy_from_slice(&tmp[2].to_le_bytes());
        res[24..32].copy_from_slice(&tmp[3].to_le_bytes());
        res[32..40].copy_from_slice(&tmp[4].to_le_bytes());
        res[40..48].copy_from_slice(&tmp[5].to_le_bytes());

        res
    }

    /// Converts an array of bytes into an `Fp6` element
    pub fn from_bytes(bytes: &[u8; 48]) -> Fp6 {
        let mut res = Fp6::zero();

        res.c0.c0 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[0..8]).unwrap(),
        ));
        res.c0.c1 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[8..16]).unwrap(),
        ));
        res.c1.c0 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[16..24]).unwrap(),
        ));
        res.c1.c1 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[24..32]).unwrap(),
        ));
        res.c2.c0 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[32..40]).unwrap(),
        ));
        res.c2.c1 = Fp::new(u64::from_le_bytes(
            <[u8; 8]>::try_from(&bytes[40..48]).unwrap(),
        ));

        res
    }

    /// Constructs an element of `Fp6` without checking that it is
    /// canonical.
    pub const fn from_raw_unchecked(value: [u64; 6]) -> Self {
        Fp6 {
            c0: Fp2 {
                c0: Fp::from_raw_unchecked(value[0]),
                c1: Fp::from_raw_unchecked(value[1]),
            },
            c1: Fp2 {
                c0: Fp::from_raw_unchecked(value[2]),
                c1: Fp::from_raw_unchecked(value[3]),
            },
            c2: Fp2 {
                c0: Fp::from_raw_unchecked(value[4]),
                c1: Fp::from_raw_unchecked(value[5]),
            },
        }
    }

    #[inline(always)]
    /// Normalizes the internal representation of an `Fp6` element
    pub fn normalize(&mut self) {
        self.c0.normalize();
        self.c1.normalize();
        self.c2.normalize();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_conditional_selection() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };
        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(7),
                c1: Fp::new(8),
            },
            c1: Fp2 {
                c0: Fp::new(9),
                c1: Fp::new(10),
            },
            c2: Fp2 {
                c0: Fp::new(11),
                c1: Fp::new(12),
            },
        };

        assert_eq!(
            ConditionallySelectable::conditional_select(&a, &b, Choice::from(0u8)),
            a
        );
        assert_eq!(
            ConditionallySelectable::conditional_select(&a, &b, Choice::from(1u8)),
            b
        );
    }

    #[test]
    fn test_equality() {
        fn is_equal(a: &Fp6, b: &Fp6) -> bool {
            let eq = a == b;
            let ct_eq = a.ct_eq(b);

            assert_eq!(eq, bool::from(ct_eq));

            eq
        }

        assert!(is_equal(
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            },
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            }
        ));

        assert!(!is_equal(
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::new(2),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            },
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            }
        ));

        assert!(!is_equal(
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(4),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            },
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            }
        ));

        assert!(!is_equal(
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(6),
                    c1: Fp::new(6),
                },
            },
            &Fp6 {
                c0: Fp2 {
                    c0: Fp::one(),
                    c1: Fp::new(2),
                },
                c1: Fp2 {
                    c0: Fp::new(3),
                    c1: Fp::new(4),
                },
                c2: Fp2 {
                    c0: Fp::new(5),
                    c1: Fp::new(6),
                },
            }
        ));
    }

    #[test]
    fn test_squaring() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::new(4009227748844085582),
                c1: Fp::new(4031374605246455920),
            },
            c1: Fp2 {
                c0: Fp::new(1533760094217716083),
                c1: Fp::new(2434700852926098351),
            },
            c2: Fp2 {
                c0: Fp::new(3070271868151135425),
                c1: Fp::new(4445614804572870244),
            },
        };
        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(1949415958821096488),
                c1: Fp::new(34803124703569509),
            },
            c1: Fp2 {
                c0: Fp::new(2372921723172571513),
                c1: Fp::new(3116692962746587027),
            },
            c2: Fp2 {
                c0: Fp::new(3662356892616824886),
                c1: Fp::new(4411799637305927293),
            },
        };

        assert_eq!(a.square(), b);
    }

    #[test]
    fn test_sqrt() {
        for _ in 0..10 {
            let a = Fp6::random(&mut thread_rng()).square();
            let b = a.sqrt_vartime().unwrap();
            assert_eq!(a, b.square());
        }
    }

    #[test]
    fn test_multiplication() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };
        let b = Fp6::one();
        let c = Fp6 {
            c0: Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };

        assert_eq!(a * b, c);

        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::new(4009227748844085582),
                c1: Fp::new(4031374605246455920),
            },
            c1: Fp2 {
                c0: Fp::new(1533760094217716083),
                c1: Fp::new(2434700852926098351),
            },
            c2: Fp2 {
                c0: Fp::new(3070271868151135425),
                c1: Fp::new(4445614804572870244),
            },
        };
        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(1949415958821096488),
                c1: Fp::new(34803124703569509),
            },
            c1: Fp2 {
                c0: Fp::new(2372921723172571513),
                c1: Fp::new(3116692962746587027),
            },
            c2: Fp2 {
                c0: Fp::new(3662356892616824886),
                c1: Fp::new(4411799637305927293),
            },
        };
        let c = Fp6 {
            c0: Fp2 {
                c0: Fp::new(3466079655920498030),
                c1: Fp::new(2417642467435273447),
            },
            c1: Fp2 {
                c0: Fp::new(44683634131777069),
                c1: Fp::new(87878120261881242),
            },
            c2: Fp2 {
                c0: Fp::new(3689005368361223881),
                c1: Fp::new(1980168815030341140),
            },
        };

        assert_eq!(a * b, c);
    }

    #[test]
    fn test_addition() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };
        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(6),
                c1: Fp::new(5),
            },
            c1: Fp2 {
                c0: Fp::new(4),
                c1: Fp::new(3),
            },
            c2: Fp2 {
                c0: Fp::new(2),
                c1: Fp::one(),
            },
        };
        let c = Fp6 {
            c0: Fp2 {
                c0: Fp::new(7),
                c1: Fp::new(7),
            },
            c1: Fp2 {
                c0: Fp::new(7),
                c1: Fp::new(7),
            },
            c2: Fp2 {
                c0: Fp::new(7),
                c1: Fp::new(7),
            },
        };

        assert_eq!(a + b, c);
    }

    #[test]
    fn test_subtraction() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::new(6),
                c1: Fp::new(5),
            },
            c1: Fp2 {
                c0: Fp::new(4),
                c1: Fp::new(3),
            },
            c2: Fp2 {
                c0: Fp::new(2),
                c1: Fp::one(),
            },
        };
        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(3),
            },
            c1: Fp2 {
                c0: Fp::new(2),
                c1: Fp::new(2),
            },
            c2: Fp2 {
                c0: Fp::one(),
                c1: Fp::one(),
            },
        };
        let c = Fp6 {
            c0: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(2),
                c1: Fp::new(1),
            },
            c2: Fp2 {
                c0: Fp::new(1),
                c1: Fp::new(0),
            },
        };

        assert_eq!(a - b, c);
    }

    #[test]
    fn test_negation() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };
        let b = Fp6 {
            c0: -Fp2 {
                c0: Fp::one(),
                c1: Fp::new(2),
            },
            c1: -Fp2 {
                c0: Fp::new(3),
                c1: Fp::new(4),
            },
            c2: -Fp2 {
                c0: Fp::new(5),
                c1: Fp::new(6),
            },
        };

        assert_eq!(-a, b);
    }

    #[test]
    fn test_inversion() {
        let a = Fp6 {
            c0: Fp2 {
                c0: Fp::new(4009227748844085582),
                c1: Fp::new(4031374605246455920),
            },
            c1: Fp2 {
                c0: Fp::new(1533760094217716083),
                c1: Fp::new(2434700852926098351),
            },
            c2: Fp2 {
                c0: Fp::new(3070271868151135425),
                c1: Fp::new(4445614804572870244),
            },
        };

        let b = Fp6 {
            c0: Fp2 {
                c0: Fp::new(2755520761612505010),
                c1: Fp::new(1048198912003742666),
            },
            c1: Fp2 {
                c0: Fp::new(4202901689791132434),
                c1: Fp::new(2790375792599204151),
            },
            c2: Fp2 {
                c0: Fp::new(122386710556669900),
                c1: Fp::new(3508229261956137550),
            },
        };

        assert_eq!(a.invert().unwrap(), b);

        assert_eq!(Fp6::one().invert().unwrap(), Fp6::one());

        assert!(bool::from(Fp6::zero().invert().is_none()));
    }

    #[test]
    fn test_lexicographic_largest() {
        assert!(!bool::from(Fp6::zero().lexicographically_largest()));
        assert!(!bool::from(Fp6::one().lexicographically_largest()));
        assert!(bool::from(
            Fp6 {
                c0: Fp2 {
                    c0: Fp::new(4167072808029173087),
                    c1: Fp::new(1618085398724889560),
                },
                c1: Fp2 {
                    c0: Fp::new(4252814005348531461),
                    c1: Fp::new(1978937931976521722),
                },
                c2: Fp2 {
                    c0: Fp::new(733612977188794891),
                    c1: Fp::new(2521078467018751009),
                },
            }
            .lexicographically_largest()
        ));
        assert!(!bool::from(
            Fp6 {
                c0: Fp2 {
                    c0: Fp::new(444552187502873250),
                    c1: Fp::new(2993539596807156777),
                },
                c1: Fp2 {
                    c0: Fp::new(358810990183514876),
                    c1: Fp::new(2632687063555524615),
                },
                c2: Fp2 {
                    c0: Fp::new(3878012018343251446),
                    c1: Fp::new(2090546528513295328),
                },
            }
            .lexicographically_largest()
        ));
        assert!(bool::from(
            Fp6 {
                c0: Fp2 {
                    c0: Fp::new(444552187502873250),
                    c1: Fp::new(2993539596807156777),
                },
                c1: Fp2::zero(),
                c2: Fp2::zero(),
            }
            .lexicographically_largest()
        ));
    }

    #[test]
    fn test_bytes() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let a = Fp6::random(&mut rng);
            let bytes = a.to_bytes();
            assert_eq!(a, Fp6::from_bytes(&bytes));
        }
    }

    #[test]
    fn test_zeroize() {
        use zeroize::Zeroize;

        let mut a = Fp6::one();
        a.zeroize();
        assert!(bool::from(a.is_zero()));
    }
}