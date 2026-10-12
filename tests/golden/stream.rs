//! A frozen copy of the generator in tests/support.rs. The hash's random inputs are this stream, so
//! it is copied rather than shared: a change to the suite's helper must not move the claim by
//! accident. splitmix64 and `f64::from_bits` only, so the stream is the same on every target.

/// splitmix64: a deterministic stream of doubles.
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

/// 2^e as a double, by its bits: `powi` reciprocates, so it returns 0 for `e <= -1024` where the
/// subnormals still reach down to -1074.
pub(crate) fn pow2(e: i32) -> f64 {
    assert!((-1074..=1023).contains(&e));
    if e >= -1022 {
        f64::from_bits(((e + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (e + 1074))
    }
}
