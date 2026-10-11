//! Helpers the test topics share: a deterministic random stream and a hasher. Nothing here is a
//! test, so the file is a `mod` of tests/main.rs like a topic but carries no `#[test]`.

use core::hash::{Hash, Hasher};

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

    /// A normal double with its leading bit at 2^e for e in [lo, hi), a full random fraction, and
    /// either sign. Its lowest bit is at or above 2^(lo - 52), so lo ≥ -44 keeps it on
    /// `Fixed256`'s grid.
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

    /// A finite double with uniformly random bits: every exponent equally likely, so subnormals,
    /// the largest magnitudes and both signs all appear.
    pub(crate) fn finite(&mut self) -> f64 {
        loop {
            let x = f64::from_bits(self.next());
            if x.is_finite() {
                return x;
            }
        }
    }

    /// Any double: one time in sixteen a NaN, +∞ or −∞, else [`Rng::finite`].
    pub(crate) fn any(&mut self) -> f64 {
        match self.next() % 48 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            2 => f64::NEG_INFINITY,
            _ => self.finite(),
        }
    }
}

pub(crate) fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
