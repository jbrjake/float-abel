//! The laws kit: what an incremental engine asks of a weight, as proptest checks that run against
//! any type.
//!
//! differential-dataflow sorts a batch's diffs with an unstable sort and adds equal keys in
//! whatever order that leaves, deletes a diff when `is_zero`, dedups by `==`, retracts by negation,
//! and evaluates `(a + b) · c` as `a·c + b·c`; DBSP asks the same of a `WSet` weight. Each of those
//! is a law, and the kit checks the laws with proptest against any [`Weight`], this crate's tiers or
//! a stranger's type, so an engine author can check a weight without reading either engine. Every
//! check takes a [`TestRunner`] and a [`Strategy`] for the weight, and reports proptest's minimal
//! failing input with the laws it broke, as a value rather than a panic:
//!
//! - [`check_abelian_group`]: associativity and commutativity, bit for bit under 1,000 shuffles of
//!   a fold; identity; and the exact inverse, `a + (−a) == zero` and `a + b − b == a`;
//! - [`check_ring`]: distributivity of a multiply, `(a + b)·k == a·k + b·k`, with its retraction
//!   form `(a − b)·k == a·k − b·k`;
//! - [`check_canonical`]: structural `Eq` is numeric equality, `a == b ⇔ (a − b).is_zero()`, and
//!   `Hash` agrees with it, so one sum reached by two histories is one structure with one hash;
//! - [`check_consolidation_invariance`]: the same multiset summed in random partitions, with the
//!   zero partials pruned between merges, lands the same bits.
//!
//! "Bit for bit" is structural `Eq`. On a canonical type that is bit equality, and a type whose
//! `Eq` is looser than its bits is what [`check_canonical`] catches.
//!
//! # Checking your own type
//!
//! Implement [`Weight`] (the group operators, the zero and its test), pick a [`Strategy`] for the
//! values your engine will see, and run the four checks. [`runner`] is a fixed-seed runner with
//! failure persistence off, so a CI run is the same run every time.
//!
//! ```
//! use core::ops::{Add, Mul, Neg, Sub};
//! use float_abel::laws::{self, proptest::prelude::*, Weight};
//!
//! /// A count that wraps: ℤ/2⁶⁴ is an abelian group, and a ring under wrapping multiply.
//! #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
//! struct Count(i64);
//!
//! impl Add for Count {
//!     type Output = Self;
//!     fn add(self, other: Self) -> Self {
//!         Count(self.0.wrapping_add(other.0))
//!     }
//! }
//! impl Sub for Count {
//!     type Output = Self;
//!     fn sub(self, other: Self) -> Self {
//!         Count(self.0.wrapping_sub(other.0))
//!     }
//! }
//! impl Neg for Count {
//!     type Output = Self;
//!     fn neg(self) -> Self {
//!         Count(self.0.wrapping_neg())
//!     }
//! }
//! impl Mul<i64> for Count {
//!     type Output = Self;
//!     fn mul(self, k: i64) -> Self {
//!         Count(self.0.wrapping_mul(k))
//!     }
//! }
//! impl Weight for Count {
//!     fn zero() -> Self {
//!         Count(0)
//!     }
//!     fn is_zero(&self) -> bool {
//!         self.0 == 0
//!     }
//! }
//!
//! let count = any::<i64>().prop_map(Count);
//! let mut runner = laws::runner(64, 1);
//! laws::check_abelian_group(&mut runner, count.clone()).unwrap();
//! laws::check_ring(&mut runner, count.clone(), any::<i64>()).unwrap();
//! laws::check_canonical(&mut runner, count.clone()).unwrap();
//! laws::check_consolidation_invariance(&mut runner, count).unwrap();
//! ```
//!
//! A control that passes would mean the kit is blind, so `tests/laws.rs` runs a naive `f64`
//! newtype through it and asserts the laws it breaks.

mod report;

use core::fmt::Debug;
use core::hash::Hash;
use core::ops::{Add, Mul, Neg, Sub};
use std::format;
use std::vec::Vec;

/// The proptest the kit is built on, so a caller's strategies and the kit agree on one version.
pub use proptest;
use proptest::arbitrary::any;
use proptest::collection::vec;
use proptest::strategy::Strategy;
use proptest::test_runner::{Config, RngSeed, TestError, TestRunner};

use crate::{Exact, Fixed, Wide};
use report::{fold, hash_of, merge_pruning, Broken, Rng};

/// The group an engine's weight must be, as the kit calls it: the operators by value, structural
/// `Eq` and `Hash`, and the zero with the test consolidation prunes on. `Clone` rather than `Copy`,
/// so a heap-backed weight can run too; this crate's tiers are `Copy` words.
pub trait Weight:
    Clone + Eq + Hash + Debug + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self>
{
    /// The identity.
    fn zero() -> Self;
    /// Whether this is the identity, as an engine's consolidation decides whether to delete it.
    fn is_zero(&self) -> bool;
}

impl<const LSB: i32, const LIMBS: usize> Weight for Fixed<LSB, LIMBS> {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        Fixed::<LSB, LIMBS>::is_zero(self)
    }
}

impl Weight for Wide {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        Wide::is_zero(self)
    }
}

impl Weight for Exact {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        Exact::is_zero(self)
    }
}

/// A deterministic runner: `cases` cases drawn from `seed`, with failure persistence off, so a CI
/// run is the same run every time and a failure writes no file. Any [`TestRunner`] serves the
/// checks; this is the one DESIGN.md § Verification asks for.
#[must_use]
pub fn runner(cases: u32, seed: u64) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(seed),
        failure_persistence: None,
        ..Config::default()
    })
}

/// What a check draws per case: the weights of a multiset, and the seed of the shuffles and
/// partitions it puts them through, so a failure replays from the value proptest reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case<T> {
    /// Between one and [`WEIGHTS`] of them, from the caller's strategy.
    pub weights: Vec<T>,
    /// Drives the shuffles, the partitions and the merge order.
    pub seed: u64,
}

/// What [`check_ring`] draws per case: the weights, and the multipliers each is scaled by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RingCase<T, K> {
    /// Between one and [`WEIGHTS`] of them, from the weight strategy.
    pub weights: Vec<T>,
    /// Between one and [`MULTIPLIERS`] of them, from the multiplier strategy.
    pub multipliers: Vec<K>,
}

/// The most weights a case draws.
pub const WEIGHTS: usize = 32;
/// The most multipliers a ring case draws.
pub const MULTIPLIERS: usize = 4;
/// Shuffles of the fold per case, DESIGN.md § The laws kit.
const SHUFFLES: usize = 1_000;
/// Random partitions of the multiset per case.
const PARTITIONS: usize = 64;
/// Histories of one sum per canonical case.
const HISTORIES: usize = 16;

fn case<T: Weight>(weight: impl Strategy<Value = T>) -> impl Strategy<Value = Case<T>> {
    (vec(weight, 1..=WEIGHTS), any::<u64>()).prop_map(|(weights, seed)| Case { weights, seed })
}

/// Checks that `weight`'s values form an abelian group under `+`, `-` and `Neg`, bit for bit:
/// every one of 1,000 shuffles of a case's fold lands the fold in the drawn order; `a + b == b + a`
/// and `(a + b) + c == a + (b + c)` over the case's weights; `a + zero == zero + a == a`;
/// `a + (−a)` is the zero by `==` and by `is_zero`; `−(−a) == a`; and `a + b − b == a`.
///
/// # Errors
///
/// proptest's minimal failing [`Case`], under a reason that names every law it broke with a
/// witness for each.
pub fn check_abelian_group<T: Weight>(
    runner: &mut TestRunner,
    weight: impl Strategy<Value = T>,
) -> Result<(), TestError<Case<T>>> {
    runner.run(&case(weight), |case| {
        let Case { weights, seed } = &case;
        let mut broken = Broken::default();
        let zero = T::zero();
        let total = fold(weights);
        let mut rng = Rng(*seed);
        let mut order = weights.clone();
        for _ in 0..SHUFFLES {
            rng.shuffle(&mut order);
            let got = fold(&order);
            if got != total {
                broken.note("associativity and commutativity", || {
                    format!("the fold of {order:?} is {got:?}; in the drawn order it is {total:?}")
                });
                break;
            }
        }
        for (i, a) in weights.iter().enumerate() {
            broken.same(
                "identity",
                || format!("{a:?} + zero"),
                &(a.clone() + zero.clone()),
                a,
            );
            broken.same(
                "identity",
                || format!("zero + {a:?}"),
                &(zero.clone() + a.clone()),
                a,
            );
            let cancelled = a.clone() + (-a.clone());
            broken.same("inverse", || format!("{a:?} + (−{a:?})"), &cancelled, &zero);
            if !cancelled.is_zero() {
                broken.note("inverse", || {
                    format!("{a:?} + (−{a:?}) is {cancelled:?}, which is_zero denies")
                });
            }
            broken.same("inverse", || format!("−(−{a:?})"), &(-(-a.clone())), a);
            for (j, b) in weights.iter().enumerate() {
                let sum = a.clone() + b.clone();
                broken.same(
                    "commutativity",
                    || format!("{a:?} + {b:?} against {b:?} + {a:?}"),
                    &sum,
                    &(b.clone() + a.clone()),
                );
                broken.same(
                    "inverse",
                    || format!("{a:?} + {b:?} − {b:?}"),
                    &(sum.clone() - b.clone()),
                    a,
                );
                let c = &weights[(i + j) % weights.len()];
                broken.same(
                    "associativity",
                    || format!("({a:?} + {b:?}) + {c:?} against {a:?} + ({b:?} + {c:?})"),
                    &(sum + c.clone()),
                    &(a.clone() + (b.clone() + c.clone())),
                );
            }
        }
        broken.report()
    })
}

/// Checks that a multiply by `multiplier`'s values distributes over the group, bit for bit:
/// `(a + b)·k == a·k + b·k`, its retraction form `(a − b)·k == a·k − b·k`, `(−a)·k == −(a·k)`
/// and `zero·k == zero`. The law differential-dataflow's join relies on, with the multiplier an
/// integer for `Fixed` and `Wide`, and an integer or an `Exact` for `Exact`.
///
/// # Errors
///
/// proptest's minimal failing [`RingCase`], under a reason with a witness.
pub fn check_ring<T, K>(
    runner: &mut TestRunner,
    weight: impl Strategy<Value = T>,
    multiplier: impl Strategy<Value = K>,
) -> Result<(), TestError<RingCase<T, K>>>
where
    T: Weight + Mul<K, Output = T>,
    K: Clone + Debug,
{
    let strategy = (vec(weight, 1..=WEIGHTS), vec(multiplier, 1..=MULTIPLIERS)).prop_map(
        |(weights, multipliers)| RingCase {
            weights,
            multipliers,
        },
    );
    runner.run(&strategy, |case| {
        let RingCase {
            weights,
            multipliers,
        } = &case;
        let mut broken = Broken::default();
        let zero = T::zero();
        for k in multipliers {
            broken.same(
                "distributivity",
                || format!("zero · {k:?}"),
                &(zero.clone() * k.clone()),
                &zero,
            );
            for a in weights {
                let scaled_a = a.clone() * k.clone();
                broken.same(
                    "distributivity",
                    || format!("(−{a:?}) · {k:?} against −({a:?} · {k:?})"),
                    &((-a.clone()) * k.clone()),
                    &(-scaled_a.clone()),
                );
                for b in weights {
                    let scaled_b = b.clone() * k.clone();
                    broken.same(
                        "distributivity",
                        || format!("({a:?} + {b:?}) · {k:?} against {a:?}·{k:?} + {b:?}·{k:?}"),
                        &((a.clone() + b.clone()) * k.clone()),
                        &(scaled_a.clone() + scaled_b.clone()),
                    );
                    broken.same(
                        "distributivity",
                        || format!("({a:?} − {b:?}) · {k:?} against {a:?}·{k:?} − {b:?}·{k:?}"),
                        &((a.clone() - b.clone()) * k.clone()),
                        &(scaled_a.clone() - scaled_b),
                    );
                }
            }
        }
        broken.report()
    })
}

/// Checks that structural `Eq` is numeric equality and `Hash` agrees with it: `zero()` is
/// `is_zero`, `a.is_zero()` iff `a == zero`, `a == b` iff `(a − b).is_zero()`, equal weights hash
/// alike, and one sum reached by two histories (a shuffled fold with one weight inserted and
/// retracted on top) is one structure with one hash.
///
/// # Errors
///
/// proptest's minimal failing [`Case`], under a reason with a witness.
pub fn check_canonical<T: Weight>(
    runner: &mut TestRunner,
    weight: impl Strategy<Value = T>,
) -> Result<(), TestError<Case<T>>> {
    runner.run(&case(weight), |case| {
        let Case { weights, seed } = &case;
        let mut broken = Broken::default();
        let zero = T::zero();
        if !zero.is_zero() {
            broken.note("canonical", || {
                format!("zero() is {zero:?}, which is_zero denies")
            });
        }
        for a in weights {
            if a.is_zero() != (*a == zero) {
                broken.note("canonical", || {
                    format!(
                        "{a:?}: is_zero says {}, == zero says {}",
                        a.is_zero(),
                        *a == zero
                    )
                });
            }
            for b in weights {
                let equal = a == b;
                let difference = a.clone() - b.clone();
                if equal != difference.is_zero() {
                    broken.note("canonical", || {
                        format!(
                            "{a:?} == {b:?} is {equal}, but {a:?} − {b:?} is {difference:?}, \
                             whose is_zero is {}",
                            difference.is_zero()
                        )
                    });
                }
                if equal && hash_of(a) != hash_of(b) {
                    broken.note("canonical", || {
                        format!("{a:?} == {b:?}, but they hash apart")
                    });
                }
            }
        }
        let total = fold(weights);
        let mut rng = Rng(*seed);
        let mut order = weights.clone();
        for _ in 0..HISTORIES {
            rng.shuffle(&mut order);
            let extra = order[rng.below(order.len())].clone();
            let detour = fold(&order) + extra.clone() - extra.clone();
            if detour != total
                || hash_of(&detour) != hash_of(&total)
                || !(detour.clone() - total.clone()).is_zero()
            {
                broken.note("canonical", || {
                    format!(
                        "the fold of {order:?} plus and minus {extra:?} is {detour:?}; in the \
                         drawn order it is {total:?}"
                    )
                });
                break;
            }
        }
        broken.report()
    })
}

/// Checks that consolidation timing cannot change a sum: a case's weights, with the retraction of
/// a random subset of them added so that partials cancel, are summed in random partitions, the
/// zero partials are pruned, the rest are merged pairwise in random order with pruning after every
/// level, and the result is the plain fold, bit for bit, 64 times per case.
///
/// # Errors
///
/// proptest's minimal failing [`Case`], under a reason with a witness.
pub fn check_consolidation_invariance<T: Weight>(
    runner: &mut TestRunner,
    weight: impl Strategy<Value = T>,
) -> Result<(), TestError<Case<T>>> {
    runner.run(&case(weight), |case| {
        let Case { weights, seed } = &case;
        let mut rng = Rng(*seed);
        let mut multiset = weights.clone();
        for w in weights {
            if rng.next() & 1 == 0 {
                multiset.push(-w.clone());
            }
        }
        let expected = fold(&multiset);
        let mut broken = Broken::default();
        for _ in 0..PARTITIONS {
            rng.shuffle(&mut multiset);
            let parts = 1 + rng.below(16);
            let mut partials = std::vec![T::zero(); parts];
            for w in &multiset {
                let p = rng.below(parts);
                partials[p] = partials[p].clone() + w.clone();
            }
            let got = merge_pruning(&mut rng, partials);
            if got != expected {
                broken.note("consolidation invariance", || {
                    format!(
                        "{parts} partitions of {multiset:?}, pruned and merged, land {got:?}; \
                         the plain fold is {expected:?}"
                    )
                });
                break;
            }
        }
        broken.report()
    })
}
