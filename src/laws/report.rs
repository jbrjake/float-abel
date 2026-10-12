//! What a check runs a case through: the seeded shuffles and partitions, the fold, the hash, the
//! pruning merge tree, and the report that names every law a case broke.

use core::fmt::Write as _;
use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::format;
use std::string::String;
use std::vec::Vec;

use proptest::test_runner::{TestCaseError, TestCaseResult};

use super::Weight;

/// splitmix64 over the seed proptest drew, so a case's shuffles and partitions are part of its
/// input and shrink with it.
pub(super) struct Rng(pub(super) u64);

impl Rng {
    pub(super) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// An index below `n`, which is at least one.
    pub(super) fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    pub(super) fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            v.swap(i, self.below(i + 1));
        }
    }
}

/// The left fold from the zero, in the given order.
pub(super) fn fold<T: Weight>(weights: &[T]) -> T {
    weights.iter().cloned().fold(T::zero(), |sum, w| sum + w)
}

pub(super) fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

/// Merges partial sums the way a merge tree does: prune the zeros, pair the rest in random order,
/// add each pair, and repeat until one is left. None left is the zero.
pub(super) fn merge_pruning<T: Weight>(rng: &mut Rng, mut partials: Vec<T>) -> T {
    loop {
        partials.retain(|p| !p.is_zero());
        if partials.len() <= 1 {
            return partials.pop().unwrap_or_else(T::zero);
        }
        rng.shuffle(&mut partials);
        partials = partials
            .chunks(2)
            .map(|pair| {
                pair.iter()
                    .cloned()
                    .reduce(|a, b| a + b)
                    .unwrap_or_else(T::zero)
            })
            .collect();
    }
}

/// The laws one case broke, with the first witness of each, so a failure names every broken law
/// at once rather than the first one met.
#[derive(Default)]
pub(super) struct Broken(Vec<(&'static str, String)>);

impl Broken {
    pub(super) fn note(&mut self, law: &'static str, witness: impl FnOnce() -> String) {
        if !self.0.iter().any(|(noted, _)| *noted == law) {
            self.0.push((law, witness()));
        }
    }

    /// Notes `law` unless `lhs == rhs`.
    pub(super) fn same<T: Weight>(
        &mut self,
        law: &'static str,
        what: impl FnOnce() -> String,
        lhs: &T,
        rhs: &T,
    ) {
        if lhs != rhs {
            self.note(law, || format!("{}: {lhs:?} is not {rhs:?}", what()));
        }
    }

    pub(super) fn report(self) -> TestCaseResult {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut text = String::from("broke");
        for (law, witness) in &self.0 {
            let _ = write!(text, "\n  {law}: {witness}");
        }
        Err(TestCaseError::fail(text))
    }
}
