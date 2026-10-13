//! Helpers the exact tiers' unit tests share. `tests/support.rs` serves the integration tests and
//! unit tests cannot see it, so the random stream is here as well.

#![allow(clippy::unreadable_literal)]

use core::hash::{Hash, Hasher};

use crate::exact::compact::Compact;

/// splitmix64: a deterministic stream for the randomised tests.
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A normal double with its leading bit at 2^e for e in [lo, hi), a full random fraction,
    /// and either sign. A pair drawn from a window of width w spans at most w + 52 bits.
    pub(crate) fn on_grid(&mut self, lo: i32, hi: i32) -> f64 {
        let e = lo + (self.next() % (hi - lo) as u64) as i32;
        let frac = self.next() >> 12;
        let x = f64::from_bits(((e + 1023) as u64) << 52 | frac);
        if self.next() & 1 == 0 {
            x
        } else {
            -x
        }
    }

    /// A finite double with uniformly random bits: every exponent equally likely.
    pub(crate) fn finite(&mut self) -> f64 {
        loop {
            let x = f64::from_bits(self.next());
            if x.is_finite() {
                return x;
            }
        }
    }
}

pub(crate) fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// `n` natively, and a hundredth of it (at least one) under miri, which interprets a limb
/// operation some thousand times slower than the machine runs it: for a sweep whose assertions
/// hold at any count.
pub(crate) const fn rounds(n: usize) -> usize {
    if cfg!(miri) {
        let cut = n / 100;
        if cut == 0 {
            1
        } else {
            cut
        }
    } else {
        n
    }
}

/// 2^e as a double, by its bits: `powi` reciprocates, so it returns 0 for `e <= -1024` where the
/// subnormals still reach down to -1074, and miri perturbs it by a few ulps on purpose, as it
/// does every operation whose result is not specified to the bit.
pub(crate) fn pow2(e: i32) -> f64 {
    assert!((-1074..=1023).contains(&e));
    if e >= -1022 {
        f64::from_bits(((e + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (e + 1074))
    }
}

pub(crate) fn c(x: f64) -> Compact {
    Compact::from_f64(x).unwrap_or_else(|e| panic!("{x:e}: {e:?}"))
}

pub(crate) fn add(a: Compact, b: Compact) -> Compact {
    a.checked_add(b)
        .unwrap_or_else(|e| panic!("{a:?} + {b:?}: {e:?}"))
}

pub(crate) fn mul(a: Compact, b: Compact) -> Compact {
    a.checked_mul(b)
        .unwrap_or_else(|e| panic!("{a:?} * {b:?}: {e:?}"))
}

pub(crate) fn product(a: f64, b: f64) -> Compact {
    Compact::from_product(a, b).unwrap_or_else(|e| panic!("{a:e} * {b:e}: {e:?}"))
}

/// `2^127 - 1`, the widest mantissa the tier holds, built through the adds that reach it:
/// `2^126 + (2^126 - 1)`, with `2^126 - 1` itself `2^126 + (-1)`.
pub(crate) fn widest() -> Compact {
    let two126 = c(pow2(126));
    let below = add(two126, c(-1.0));
    assert_eq!(below.parts(), (u64::MAX, (1 << 62) - 1, 0));
    let w = add(two126, below);
    assert_eq!(w.parts(), (u64::MAX, u64::MAX >> 1, 0));
    w
}
