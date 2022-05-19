// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module implements lookup tables for points in Affine coordinates.
//!
//! Adapted from https://github.com/RustCrypto/elliptic-curves

use crate::{AffinePoint, ProjectivePoint, Scalar};
use crate::{MINUS_SHIFT_POINT_ARRAY, SHIFT_POINT};

use core::ops::Mul;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

/// A lookup table for storing the precomputed values
/// `[P, 2P, 3P, ..., NP]` in affine coordinates.
#[derive(Clone, Copy, Debug)]
pub struct LookupTable<const N: usize>(pub(crate) [AffinePoint; N]);

impl<const N: usize> From<&ProjectivePoint> for LookupTable<N> {
    fn from(p: &ProjectivePoint) -> Self {
        let mut points = [*p; N];
        for i in 1..N {
            points[i] = p + points[i - 1];
        }

        let mut points_affine = [AffinePoint::identity(); N];
        ProjectivePoint::batch_normalize(&points, &mut points_affine);

        Self(points_affine)
    }
}

impl<const N: usize> From<ProjectivePoint> for LookupTable<N> {
    fn from(p: ProjectivePoint) -> Self {
        Self::from(&p)
    }
}

impl<const N: usize> From<AffinePoint> for LookupTable<N> {
    fn from(p: AffinePoint) -> Self {
        Self::from(&ProjectivePoint::from(&p))
    }
}

impl<const N: usize> From<&AffinePoint> for LookupTable<N> {
    fn from(p: &AffinePoint) -> Self {
        Self::from(&ProjectivePoint::from(p))
    }
}

impl<const N: usize> Zeroize for LookupTable<N> {
    fn zeroize(&mut self) {
        for point in self.0.iter_mut() {
            point.zeroize();
        }
    }
}

impl<const N: usize> LookupTable<N> {
    /// Given an `i8` x value, returns x.P.
    // To do so, we first compute |x|.P, and then conditionally
    // negate the result based on the sign of x.
    pub(crate) fn get_point(&self, x: i8) -> AffinePoint {
        debug_assert!(x >= -(N as i8));
        debug_assert!(x <= N as i8);

        // Compute xabs = |x|
        let xmask = x >> 7;
        let xabs = (x + xmask) ^ xmask;

        // Get an array element in constant time
        let mut t = AffinePoint::identity();
        for j in 1..N + 1 {
            let c = (xabs as u8).ct_eq(&(j as u8));
            t.conditional_assign(&self.0[j - 1], c);
        }
        // Now t == |x| * p.

        let neg_mask = Choice::from((xmask & 1) as u8);
        t.conditional_assign(&-t, neg_mask);
        // Now t == x * p.

        t
    }

    /// Given an `i8` x value, returns x.P.
    // To do so, we first compute |x|.P, and then conditionally
    // negate the result based on the sign of x.
    pub(crate) fn get_point_vartime(&self, x: i8) -> AffinePoint {
        debug_assert!(x >= -(N as i8));
        debug_assert!(x <= N as i8);

        // Compute xabs = |x|
        let xmask = x >> 7;
        let xabs = (x + xmask) ^ xmask;

        // Get an array element
        let mut t = AffinePoint::identity();
        for j in 1..N + 1 {
            if xabs as usize == j {
                t = self.0[j - 1];
            }
        }
        // Now t == |x| * p.

        if xmask & 1 == 1 {
            t = -t;
        }
        // Now t == x * p.

        t
    }
}

/// A list of `LookupTable` of multiples of a point `P` to perform
/// efficient scalar multiplication with the Pippenger's algorith,
/// https://cr.yp.to/papers/pippenger.pdfm.
///
/// Creating a base point table is costly, and hence should be
/// done in the purpose of being used more than once.
#[derive(Clone, Debug)]
pub struct BasePointTable(pub [LookupTable<8>; 32]);

impl From<AffinePoint> for BasePointTable {
    fn from(p: AffinePoint) -> Self {
        Self::create(&ProjectivePoint::from(&p))
    }
}

impl From<&AffinePoint> for BasePointTable {
    fn from(p: &AffinePoint) -> Self {
        Self::create(&ProjectivePoint::from(p))
    }
}

impl From<ProjectivePoint> for BasePointTable {
    fn from(p: ProjectivePoint) -> Self {
        Self::create(&p)
    }
}

impl From<&ProjectivePoint> for BasePointTable {
    fn from(p: &ProjectivePoint) -> Self {
        Self::create(p)
    }
}

impl BasePointTable {
    /// Returns a precomputed table of multiples of a given point.
    pub fn create(basepoint: &ProjectivePoint) -> Self {
        let mut table = BasePointTable([LookupTable::from(basepoint); 32]);
        let mut point = *basepoint;
        for i in 1..32 {
            point = point.double_multi(8);
            table.0[i] = LookupTable::from(&point);
        }

        table
    }

    /// Get the basepoint of this table.
    pub fn get_basepoint(&self) -> AffinePoint {
        self.0[0].get_point(1)
    }

    /// Get the basepoint of this table.
    pub fn get_basepoint_vartime(&self) -> AffinePoint {
        self.0[0].get_point_vartime(1)
    }

    /// Performs a mixed scalar multiplication from `by`
    /// given as byte representation of a `Scalar` element, by
    /// using internally the Pippenger's algorithm.
    #[inline]
    pub fn multiply(&self, scalar: &[u8; 32]) -> ProjectivePoint {
        let a = Scalar::bytes_to_radix_16(scalar);

        let tables = &self.0;
        let mut acc = SHIFT_POINT;

        for i in (0..64).filter(|x| x % 2 == 1) {
            acc = acc.add_mixed_unchecked(&tables[i / 2].get_point(a[i]));
        }

        acc = acc.double_multi(4);

        for i in (0..64).filter(|x| x % 2 == 0) {
            acc = acc.add_mixed_unchecked(&tables[i / 2].get_point(a[i]));
        }

        acc.add_mixed_unchecked(&MINUS_SHIFT_POINT_ARRAY[4])
    }

    /// Performs a mixed scalar multiplication from `by`
    /// given as byte representation of a `Scalar` element, by
    /// using internally the Pippenger's algorithm.
    ///
    /// **This operation is variable time with respect
    /// to the scalar.** If the scalar is fixed,
    /// this operation is effectively constant time.
    #[inline]
    pub fn multiply_vartime(&self, scalar: &[u8; 32]) -> ProjectivePoint {
        let a = Scalar::bytes_to_radix_16(scalar);

        let tables = &self.0;
        let mut acc = SHIFT_POINT;

        for i in (0..64).filter(|x| x % 2 == 1) {
            acc = acc.add_mixed_unchecked(&tables[i / 2].get_point(a[i]));
        }

        acc = acc.double_multi(4);

        for i in (0..64).filter(|x| x % 2 == 0) {
            acc = acc.add_mixed_unchecked(&tables[i / 2].get_point(a[i]));
        }

        acc.add_mixed_unchecked(&MINUS_SHIFT_POINT_ARRAY[4])
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a BasePointTable {
    type Output = ProjectivePoint;

    fn mul(self, scalar: &'b Scalar) -> Self::Output {
        self.multiply(&scalar.to_bytes())
    }
}

impl_binops_multiplicative_mixed!(BasePointTable, Scalar, ProjectivePoint);
