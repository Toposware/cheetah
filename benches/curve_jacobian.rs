// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand_core::OsRng;

#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;

extern crate cheetah;

use cheetah::Scalar;
use cheetah::{AffinePoint, JacobianPoint};

static BATCH_SIZES: [u32; 5] = [1, 10, 100, 1000, 10000];

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = OsRng;
    let p = JacobianPoint::random(&mut rng);
    let p2 = JacobianPoint::random(&mut rng);
    let p2_affine = AffinePoint::from(&p2);
    let pow = Scalar::random(&mut rng).to_bytes();
    let pow2 = Scalar::random(&mut rng).to_bytes();

    let uncompressed_encoding = p.to_uncompressed();
    let compressed_encoding = p.to_compressed();

    c.bench_function("Jacobian addition", |bench| {
        bench.iter(|| black_box(p) + black_box(p2))
    });

    c.bench_function("Jacobian mixed addition", |bench| {
        bench.iter(|| black_box(p) + black_box(p2_affine))
    });

    c.bench_function("Jacobian unchecked mixed addition", |bench| {
        bench.iter(|| black_box(p).add_mixed_unchecked(black_box(&p2_affine)))
    });

    c.bench_function("Jacobian subtraction", |bench| {
        bench.iter(|| black_box(p) - black_box(p2))
    });

    c.bench_function("Jacobian doubling", |bench| {
        bench.iter(|| black_box(p).double())
    });

    c.bench_function("Jacobian unchecked doubling", |bench| {
        bench.iter(|| black_box(p).double_unchecked())
    });

    c.bench_function("Jacobian scalar multiplication (variable base)", |bench| {
        bench.iter(|| JacobianPoint::multiply(black_box(&p), black_box(&pow)))
    });

    c.bench_function(
        "Jacobian scalar multiplication (variable base) - variable time",
        |bench| bench.iter(|| JacobianPoint::multiply_vartime(black_box(&p), black_box(&pow))),
    );

    c.bench_function(
        "Jacobian double scalar multiplication (variable base)",
        |bench| {
            bench.iter(|| {
                JacobianPoint::multiply_double(
                    black_box(&p),
                    black_box(&p2),
                    black_box(&pow),
                    black_box(&pow2),
                )
            })
        },
    );

    c.bench_function(
        "Jacobian double scalar multiplication (variable base) - variable time",
        |bench| {
            bench.iter(|| {
                JacobianPoint::multiply_double_vartime(
                    black_box(&p),
                    black_box(&p2),
                    black_box(&pow),
                    black_box(&pow2),
                )
            })
        },
    );

    c.bench_function(
        "Jacobian double scalar multiplication with basepoint - variable time",
        |bench| {
            bench.iter(|| {
                JacobianPoint::multiply_double_with_basepoint_vartime(
                    black_box(&p),
                    black_box(&pow),
                    black_box(&pow2),
                )
            })
        },
    );

    c.bench_function("Jacobian uncompressed encoding", |bench| {
        bench.iter(|| JacobianPoint::to_uncompressed(black_box(&p)))
    });

    c.bench_function("Jacobian uncompressed decoding", |bench| {
        bench.iter(|| JacobianPoint::from_uncompressed(black_box(&uncompressed_encoding)))
    });

    c.bench_function("Jacobian compressed encoding", |bench| {
        bench.iter(|| JacobianPoint::to_compressed(black_box(&p)))
    });

    c.bench_function("Jacobian compressed decoding", |bench| {
        bench.iter(|| JacobianPoint::from_compressed(black_box(&compressed_encoding)))
    });

    c.bench_function("Jacobian curve check", |bench| {
        bench.iter(|| JacobianPoint::is_on_curve(black_box(&p)))
    });

    c.bench_function("Jacobian subgroup check", |bench| {
        bench.iter(|| JacobianPoint::is_torsion_free(black_box(&p)))
    });

    c.bench_function("Jacobian points equality", |bench| {
        bench.iter(|| black_box(p) == black_box(p))
    });

    c.bench_function("Jacobian conversion to affine", |bench| {
        bench.iter(|| AffinePoint::from(black_box(&p)))
    });

    let batch_str = "Jacobian batch normalize to affine - ".to_string();
    for &batch_size in BATCH_SIZES.iter() {
        let name = batch_str.clone() + &batch_size.to_string();
        c.bench_function(&name, |bench| {
            let jacobian_points = vec![JacobianPoint::random(&mut rng); batch_size as usize];
            let mut affine_points = vec![AffinePoint::identity(); batch_size as usize];
            bench.iter(|| {
                JacobianPoint::batch_normalize(
                    black_box(&jacobian_points),
                    black_box(&mut affine_points),
                )
            })
        });
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark);
criterion_main!(benches);