// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides an implementation of the STARK-friendly elliptic
//! curve E(Fp6): y^2 = x^3 + x + B with
//! p = 2^64 - 2^32 + 1
//!   = 18446744069414584321
//! B = u + 395
//! where u^6 - 7 = 0
//!
//! Portions of the codebase have been adapted from https://github.com/zkcrypto/bls12_381

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(clippy::many_single_char_names)]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate lazy_static;

// Re-export group (which re-exports ff)
pub use group;

#[macro_use]
mod utils;

mod constants;
mod lookup;
mod naf_lookup;

mod curve;
mod fp;
mod fp3;
mod fp6;
mod scalar;

pub use scalar::Scalar;

pub use fp::Fp;
pub use fp6::Fp6;

pub(crate) use constants::SHIFT_POINT_MODIFIED_JACOBIAN;
pub use constants::{
    BASEPOINT_LOOKUP, BASEPOINT_TABLE, MINUS_SHIFT_POINT_ARRAY, SHIFT_POINT_AFFINE,
    SHIFT_POINT_JACOBIAN, SHIFT_POINT_PROJECTIVE,
};

pub use lookup::{BasePointTable, LookupTable};
pub use naf_lookup::NafLookupTable;

pub(crate) use curve::ModifiedJacobianPoint;
pub use curve::{
    AffinePoint, CompressedPoint, JacobianPoint, ProjectivePoint, UncompressedPoint, B,
};

/// Helper methods for arithmetic reduction of integers mod p
pub mod fp_arith_utils {
    pub use crate::fp::{reduce_u128, reduce_u96};
}
