# Cheetah 🐆

This crates provide an implementation of the Cheetah curve over the field extension $\mathbb{F}_{p^6}$, with p = 2<sup>64</sup> - 2<sup>32</sup> + 1.

* This implementation does not require the Rust standard library
* Arithmetic operations are all constant time unless "_vartime" is explicited mentioned

**WARNING:** This is an ongoing, prototype implementation subject to changes. In particular, it has not been audited and may contain bugs and security flaws. This implementation is NOT ready for production use.

## Features

* `serialize` (on by default): Enables Serde serialization

## Description

Cheetah is a STARK-friendly elliptic curve defined over a sextic extension of $\mathbb{F}_p$, p = 2<sup>64</sup> - 2<sup>32</sup> + 1 defined by
`E: y^2 = x^3 + x + B` with 
`B = 8751648879042009540*u^5 + 9152663345541292493*u^4 + 11461655964319546493*u^3 + 18139339604299112321*u^2 + 11484739320139982777*u + 13239264519837388909`
where
- `u^6 - 7 = 0` is the polynomial defining $\mathbb{F}_{p^6} / \mathbb{F}_p$

Cheetah defines a subgroup G of prime order `q = 0x169e15bdd51495399677863fe04d28d8a0a87b464e77710691776f9d741b35eb` of 253-bits.

The extension $\mathbb{F}_{p^6}$ has been specifically constructed with a sparse polynomial of the form `X^6 - A`, where `A` is a small quadratic and cubic non-residue. The current implementation may however not be fully optimal with respect to the number of multiplications in the base field.

The Cheetah curve has been generated with the Sagemath utility script `sextic_search.sage` available [here](https://github.com/Toposware/cheetah_evidence).


## Curve security

Elliptic curves based on extension fields may suffer from specific attacks that do not apply to common elliptic curves constructed over large prime fields and may outperform regular Pollard-Rho attacks, and hence require more scrutiny when evaluating their estimated security level. To verify the security level
of Cheetah against generic attacks as well as cover and decomposition attacks, please use the Sagemath utility script `verify.sage` available
[here](https://github.com/Toposware/cheetah_evidence).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
