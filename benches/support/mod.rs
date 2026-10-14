//! The fixtures the bench targets share: the suite's deterministic stream and the design's
//! magnitude bands. A directory module rather than a `benches/*.rs` file, so cargo never discovers
//! it as a bench target of its own. Each target is its own crate and takes the fixtures it needs,
//! hence the `dead_code` allowances.

// The fixtures build doubles from their bits and move bits between integer widths on purpose, and
// the SUM's amount is a count of units below 2²⁰, exact as a double.
#![allow(
    dead_code,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

/// Elements in a fixture: 2²⁰, so a run is a few milliseconds and a rate has six digits.
pub(crate) const N: usize = 1 << 20;

/// The seed every band is drawn with, so a re-run on another machine folds the same multiset.
const SEED: u64 = 0xBE4C_4D41_524B;

/// splitmix64, the suite's stream.
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
    /// either sign.
    pub(crate) fn on_grid(&mut self, lo: i32, hi: i32) -> f64 {
        self.on_grid_with(lo, hi, 53)
    }

    /// As [`Rng::on_grid`], with the mantissa cut to `bits` significant bits, so the lowest set
    /// bit lies at or above 2^(e − bits + 1).
    pub(crate) fn on_grid_with(&mut self, lo: i32, hi: i32, bits: u32) -> f64 {
        let e = lo + (self.next() % (hi - lo) as u64) as i32;
        let frac = (self.next() >> 12) & !((1u64 << (53 - bits)) - 1);
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
}

/// `N` doubles with top bits in [2^lo, 2^hi): the design's same-scale band at (−20, 20) and its
/// wide band at (−300, 300).
pub(crate) fn fixture(lo: i32, hi: i32) -> Vec<f64> {
    let mut rng = Rng(SEED);
    (0..N).map(|_| rng.on_grid(lo, hi)).collect()
}

/// `N` finite doubles with uniformly random bits: the adversarial band, where a 127-bit word
/// cannot hold two values 128 binades apart and a fold meets such a pair at once.
pub(crate) fn adversarial() -> Vec<f64> {
    let mut rng = Rng(SEED);
    (0..N).map(|_| rng.finite()).collect()
}

/// `N` factor pairs on the same-scale band with 26-bit mantissas, so every exact product has its
/// lowest bit at or above 2⁻⁹⁰ and `Fixed256`, whose unit is 2⁻⁹⁶, accepts it; the magnitudes stay
/// under 2⁴⁰, inside every tier's ceiling. One corpus for every tier's `deposit_products`.
pub(crate) fn product_fixture() -> (Vec<f64>, Vec<f64>) {
    let mut rng = Rng(SEED);
    (0..N)
        .map(|_| (rng.on_grid_with(-20, 20, 26), rng.on_grid_with(-20, 20, 26)))
        .unzip()
}

/// The accounts the differential-dataflow fixture spreads its rows over.
pub(crate) const ACCOUNTS: u64 = 16;

/// One row of the differential-dataflow fixture: an account and an amount as a signed count of
/// units of 2⁻²⁰.
pub(crate) type Row = (u32, i64);

/// `n` rows over sixteen accounts, each amount a signed count of units of 2⁻²⁰ below 2²⁰ in
/// magnitude: the integration harness's same-scale fixture (tests/differential.rs), which `Exact`
/// carries as the double and `isize` as the count, so both diffs fold one multiset.
pub(crate) fn units(n: u64) -> Vec<Row> {
    (0..n)
        .map(|i| {
            let account = ((i * 7919) % ACCOUNTS) as u32;
            (account, nth_units(i))
        })
        .collect()
}

/// The `i`-th amount of the fixture, a signed count of units of 2⁻²⁰ below 2²⁰ in magnitude. The
/// multiplier is odd, so the first 2²¹ amounts are distinct and the sequence then repeats.
pub(crate) fn nth_units(i: u64) -> i64 {
    ((i.wrapping_mul(2_654_435_761)) % (1 << 21)) as i64 - (1 << 20)
}

/// `units × 2⁻²⁰` as a double, exactly: the power of two by its bits, never `powi`.
pub(crate) fn amount(units: i64) -> f64 {
    units as f64 * f64::from_bits(((-20i64 + 1023) as u64) << 52)
}
