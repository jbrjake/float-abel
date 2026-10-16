//! The architecture claim, DESIGN.md § Bit-identical across architectures: one SHA-256 over every
//! tier's canonical bytes and reads on fixed inputs, equal to tests/data/golden.sha256 on every
//! target .github/workflows/golden.yml runs. The inputs are the rounding and conformance vector
//! sets and 10⁴ seeded random multisets from the stream frozen in stream.rs. Each multiset goes
//! through every tier: `Wide` with its group ops, its reads and its view as a `Binned`; a `Binned`
//! fold with its bins and the `Wide` it converts to, since `Binned` has no wire form; `Fixed256`
//! with every input accepted or refused by name; `Exact` with every input accepted, in either form
//! by its bytes; the conversions between them; the 106-bit product on each; the bulk entries,
//! `extend_from_slice` on `Wide`, `Fixed256` and `Binned` and the dot product of the multiset with
//! its own reverse through `deposit_products` on each, which run whatever kernel this target
//! dispatches to behind the `simd` feature, and through `Exact`'s product deposits one pair at a
//! time, a fold of products that promotes.
//! Every integer enters the digest little-endian, every double as its
//! bits and every refusal as one tag byte, so a difference on a big-endian target is the crate's,
//! never this file's. Every test is named `golden_*`.
//!
//! The committed hash is the claim, computed once on aarch64 macOS. A mismatch on any target is a
//! finding to run down, never a reason to re-bless. When the inputs or the record layout change on
//! purpose, the failing assertion prints the new hash to commit.

// The digest compares doubles bit for bit and moves bits between integer widths on purpose.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

mod stream;
mod vectors;

use float_abel::{Binned, Exact, Fixed256, OutOfWindow, PastCap, Wide};
use sha2::{Digest as _, Sha256};

use crate::stream::Rng;

/// One SHA-256 in lowercase hex and a newline, nothing else.
const CLAIM: &str = include_str!("../data/golden.sha256");
const SEED: u64 = 0x006F_6C64_656E;
const MULTISETS: u64 = 10_000;
/// The doubles across the 10⁴ multisets, and the bytes the digest absorbed. Both follow from the
/// seed and the record layout, and both are asserted before the hash because a different count of
/// records is a more legible finding than a different hash.
const ELEMENTS: u64 = 325_745;
const BYTES: u64 = 74_365_506;
/// The batch tier's width here: three bins, the design's default.
const K: usize = 3;

/// One byte opens every record, so two different sequences of records never share a stream.
mod tag {
    /// A multiset or a conformance case: its length, then its doubles.
    pub(crate) const INPUTS: u8 = 0x10;
    /// A rounding vector: sign, exponent, then its mantissa limbs.
    pub(crate) const READ: u8 = 0x11;
    /// A quotient vector: its divisor, then its vector.
    pub(crate) const QUOTIENT: u8 = 0x12;
    pub(crate) const WIDE: u8 = 0x20;
    pub(crate) const FIXED: u8 = 0x21;
    pub(crate) const EXACT: u8 = 0x22;
    pub(crate) const BINNED: u8 = 0x23;
    /// An `Exact` in its heap form: its `encode` bytes, the tag then the block, and its reads.
    pub(crate) const EXACT_HEAP: u8 = 0x24;
    /// A lone double read, such as a quotient.
    pub(crate) const F64: u8 = 0x30;
}

/// A refusal's byte: `Ok` is zero, the window's three reasons 1 to 3, the cap's three 4 to 6.
trait Refusal {
    fn tag(&self) -> u8;
}

impl Refusal for OutOfWindow {
    fn tag(&self) -> u8 {
        match self {
            Self::NotFinite => 1,
            Self::OffGrid => 2,
            Self::AboveCeiling => 3,
        }
    }
}

impl Refusal for PastCap {
    fn tag(&self) -> u8 {
        match self {
            Self::Width => 4,
            Self::Exponent => 5,
            Self::Count => 6,
            Self::Limbs => 7,
        }
    }
}

/// The digest and a count of what it absorbed.
struct Transcript {
    sha: Sha256,
    bytes: u64,
}

impl Transcript {
    fn new() -> Self {
        Self {
            sha: Sha256::new(),
            bytes: 0,
        }
    }

    fn put(&mut self, bytes: &[u8]) {
        self.sha.update(bytes);
        self.bytes += bytes.len() as u64;
    }

    fn tag(&mut self, t: u8) {
        self.put(&[t]);
    }

    fn u64(&mut self, x: u64) {
        self.put(&x.to_le_bytes());
    }

    fn i64(&mut self, x: i64) {
        self.put(&x.to_le_bytes());
    }

    fn f64(&mut self, x: f64) {
        self.u64(x.to_bits());
    }

    /// A tier's three reads after its bytes: `to_f64`, `try_to_f64` as a presence byte and the
    /// bits, and `to_f32`.
    fn reads(&mut self, value: f64, exact: Option<f64>, single: f32) {
        self.f64(value);
        match exact {
            None => self.tag(0),
            Some(v) => {
                self.tag(1);
                self.f64(v);
            }
        }
        self.put(&single.to_bits().to_le_bytes());
    }

    /// A result's tag byte; the value when there is one.
    fn result<T, E: Refusal>(&mut self, r: Result<T, E>) -> Option<T> {
        match r {
            Ok(v) => {
                self.tag(0);
                Some(v)
            }
            Err(e) => {
                self.tag(e.tag());
                None
            }
        }
    }

    fn wide(&mut self, w: &Wide) {
        self.tag(tag::WIDE);
        self.put(&w.to_bytes());
        self.reads(w.to_f64(), w.try_to_f64(), w.to_f32());
    }

    fn fixed(&mut self, f: &Fixed256) {
        self.tag(tag::FIXED);
        let bytes: [u8; Fixed256::BYTES] = f.to_bytes();
        self.put(&bytes);
        self.reads(f.to_f64(), f.try_to_f64(), f.to_f32());
    }

    /// Either form by its own tag and its `encode` bytes, which for the compact form are
    /// `to_bytes`.
    fn exact(&mut self, e: &Exact) {
        self.tag(if e.is_compact() {
            tag::EXACT
        } else {
            tag::EXACT_HEAP
        });
        self.put(&e.encode());
        self.reads(e.to_f64(), e.try_to_f64(), e.to_f32());
    }

    /// The bins themselves, then the `Wide` they convert to: `Binned` has no wire form.
    fn binned(&mut self, b: &Binned<K>) {
        self.tag(tag::BINNED);
        self.put(&b.index().to_le_bytes());
        for &limb in b.limbs() {
            self.u64(limb);
        }
        for &carry in b.carries() {
            self.i64(carry);
        }
        let (n, p, q) = b.specials();
        self.i64(n);
        self.i64(p);
        self.i64(q);
        self.wide(&b.into_wide());
    }

    fn inputs(&mut self, xs: &[f64]) {
        self.tag(tag::INPUTS);
        self.u64(xs.len() as u64);
        for &x in xs {
            self.f64(x);
        }
    }

    fn finish(self) -> (String, u64) {
        (format!("{:x}", self.sha.finalize()), self.bytes)
    }
}

/// What the transcript saw, for the assertions that run before the hash is compared.
#[derive(Debug, Default)]
struct Stats {
    reads: usize,
    quotients: usize,
    cases: usize,
    multisets: u64,
    elements: u64,
    /// Inputs `Fixed256` refused one at a time and in bulk, products `Wide`, `Fixed256` and
    /// `Binned` refused, deposits that left `Exact` in its heap form, product folds that did, and
    /// heap values carrying a dyadic special amount: each path must have been taken, so each is
    /// asserted positive. A product deposit on `Exact::ZERO` never promotes (two doubles' product
    /// fits the width) and is not counted.
    fixed_refused: u64,
    fixed_bulk_refused: u64,
    wide_products_refused: u64,
    fixed_products_refused: u64,
    binned_products_refused: u64,
    exact_promoted: u64,
    exact_dot_promoted: u64,
    exact_dyadic: u64,
    bytes: u64,
}

/// Every rounding vector on every tier that holds it, and every quotient's `div_f64` likewise.
fn rounding(t: &mut Transcript, s: &mut Stats) {
    let (reads, quotients) = vectors::rounding();
    for v in &reads {
        t.tag(tag::READ);
        t.tag(u8::from(v.negative));
        t.put(&v.exp.to_le_bytes());
        t.u64(v.mant.len() as u64);
        for &limb in &v.mant {
            t.u64(limb);
        }
        let mut held = 0;
        if let Some(w) = vectors::wide_of(v) {
            t.wide(&w);
            held += 1;
        }
        if let Some(f) = vectors::fixed_of(v) {
            t.fixed(&f);
            held += 1;
        }
        if let Some(e) = vectors::exact_of(v) {
            t.exact(&e);
            held += 1;
        }
        assert!(held > 0, "{}: no tier holds it", v.name);
        s.reads += 1;
    }
    for q in &quotients {
        t.tag(tag::QUOTIENT);
        t.u64(q.n);
        let mut held = 0;
        for quotient in [
            vectors::wide_of(&q.v).map(|w| w.div_f64(q.n)),
            vectors::fixed_of(&q.v).map(|f| f.div_f64(q.n)),
            vectors::exact_of(&q.v).map(|e| e.div_f64(q.n)),
        ]
        .into_iter()
        .flatten()
        {
            t.tag(tag::F64);
            t.f64(quotient);
            held += 1;
        }
        assert!(held > 0, "{}: no tier holds it", q.v.name);
        s.quotients += 1;
    }
}

/// Every conformance case through `Wide`, `Binned` and the checked `Exact` fold.
fn conformance(t: &mut Transcript, s: &mut Stats) {
    for (name, inputs) in vectors::conformance() {
        t.inputs(&inputs);
        let wide_sum: Wide = inputs.iter().copied().map(Wide::from).sum();
        t.wide(&wide_sum);
        let mut bins = Binned::<K>::ZERO;
        bins.extend_from_slice(&inputs);
        t.binned(&bins);
        let mut exact_sum = Exact::ZERO;
        for &x in &inputs {
            t.result(exact_sum.checked_deposit(x));
            s.exact_promoted += u64::from(!exact_sum.is_compact());
        }
        t.exact(&exact_sum);
        assert_ne!(name, "", "a conformance case has a name");
        s.cases += 1;
    }
}

/// The multisets cycle through four kinds: any double, any finite double, `Fixed256`'s window, and
/// a band every tier holds whole (a sum of 64 values with top bits below 2^36 and lowest bits at or
/// above 2^-82 spans under 127 bits).
fn multiset(rng: &mut Rng, round: u64) -> Vec<f64> {
    let len = 1 + (rng.next() % 64) as usize;
    (0..len)
        .map(|_| match round % 4 {
            0 => rng.any(),
            1 => rng.finite(),
            2 => rng.on_grid(-44, 95),
            _ => rng.on_grid(-30, 36),
        })
        .collect()
}

/// One multiset through every tier. `scale` stays within ±7 so that `Wide * scale` is inside the
/// headroom `Wide` asserts on every build: 64 doubles below 2^1024, times 7, is under 2^1037.
fn absorb(t: &mut Transcript, s: &mut Stats, rng: &mut Rng, xs: &[f64]) {
    let scale = (rng.next() % 15) as i64 - 7;
    let divisor = 1 + rng.next() % 1000;
    let factor = Exact::from(rng.on_grid(-10, 10));
    let (first, last) = (xs[0], xs[xs.len() - 1]);
    t.inputs(xs);

    // Wide: the fold, its group ops and ring scaling, a quotient, and its view as a Binned.
    let wide_sum: Wide = xs.iter().copied().map(Wide::from).sum();
    t.wide(&wide_sum);
    // The same multiset through the bulk entry: whatever kernel runs here must land the fold's bits.
    let mut wide_bulk = Wide::ZERO;
    wide_bulk.extend_from_slice(xs);
    t.wide(&wide_bulk);
    t.wide(&(-wide_sum));
    t.wide(&(wide_sum - Wide::from(first)));
    t.wide(&(wide_sum * scale));
    t.tag(tag::F64);
    t.f64(wide_sum.div_f64(divisor));
    t.binned(&wide_sum.to_binned::<K>());

    // Binned: the batch fold.
    let mut bins = Binned::<K>::ZERO;
    bins.extend_from_slice(xs);
    t.binned(&bins);

    // Fixed256: each input accepted or refused by name, then the state, its scaling, a quotient,
    // and its conversions.
    let mut fixed_sum = Fixed256::ZERO;
    for &x in xs {
        if t.result(fixed_sum.try_deposit(x)).is_none() {
            s.fixed_refused += 1;
        }
    }
    t.fixed(&fixed_sum);
    let mut fixed_bulk = Fixed256::ZERO;
    match t.result(fixed_bulk.extend_from_slice(xs)) {
        Some(()) => t.fixed(&fixed_bulk),
        None => s.fixed_bulk_refused += 1,
    }
    t.fixed(&(fixed_sum * scale));
    t.tag(tag::F64);
    t.f64(fixed_sum.div_f64(divisor));
    t.wide(&Wide::from(fixed_sum));
    if let Some(e) = t.result(Exact::try_from(&fixed_sum)) {
        t.exact(&e);
    }

    // Exact: every input's own state, the fold through the checked API, the ring, a quotient, and
    // the conversions to and from Wide.
    let mut exact_sum = Exact::ZERO;
    for &x in xs {
        t.exact(&Exact::from(x));
        t.result(exact_sum.checked_deposit(x));
        s.exact_promoted += u64::from(!exact_sum.is_compact());
    }
    t.exact(&exact_sum);
    if let Some(e) = t.result(exact_sum.checked_mul_i64(scale)) {
        t.exact(&e);
    }
    if let Some(e) = t.result(exact_sum.checked_mul(&factor)) {
        // A non-integer factor on a special count is a dyadic amount, which only the heap form
        // holds.
        s.exact_dyadic += u64::from(!e.is_compact() && e.specials().is_none());
        t.exact(&e);
    }
    t.tag(tag::F64);
    t.f64(exact_sum.div_f64(divisor));
    if let Some(w) = t.result(Wide::try_from(&exact_sum)) {
        t.wide(&w);
    }
    if let Some(e) = t.result(Exact::try_from(&wide_sum)) {
        t.exact(&e);
    }

    products(t, s, xs, first, last);

    s.multisets += 1;
    s.elements += xs.len() as u64;
}

/// The products of one multiset: the 106-bit product of its first and last inputs on every tier,
/// and its dot product with its own reverse through each tier's bulk entry, or the refusal, and
/// through `Exact` one product deposit per pair, a fold of products that promotes on mixed
/// magnitudes and holds every pair.
fn products(t: &mut Transcript, s: &mut Stats, xs: &[f64], first: f64, last: f64) {
    t.exact(&Exact::product(first, last));
    let mut wide_product = Wide::ZERO;
    match t.result(wide_product.deposit_product(first, last)) {
        Some(()) => t.wide(&wide_product),
        None => s.wide_products_refused += 1,
    }
    let mut fixed_product = Fixed256::ZERO;
    match t.result(fixed_product.deposit_product(first, last)) {
        Some(()) => t.fixed(&fixed_product),
        None => s.fixed_products_refused += 1,
    }
    let mut binned_product = Binned::<K>::ZERO;
    match t.result(binned_product.deposit_product(first, last)) {
        Some(()) => t.binned(&binned_product),
        None => s.binned_products_refused += 1,
    }
    let reversed: Vec<f64> = xs.iter().rev().copied().collect();
    let mut wide_dot = Wide::ZERO;
    match t.result(wide_dot.deposit_products(xs, &reversed)) {
        Some(()) => t.wide(&wide_dot),
        None => s.wide_products_refused += 1,
    }
    let mut fixed_dot = Fixed256::ZERO;
    match t.result(fixed_dot.deposit_products(xs, &reversed)) {
        Some(()) => t.fixed(&fixed_dot),
        None => s.fixed_products_refused += 1,
    }
    let mut dot = Binned::<K>::ZERO;
    match t.result(dot.deposit_products(xs, &reversed)) {
        Some(()) => t.binned(&dot),
        None => s.binned_products_refused += 1,
    }
    let mut exact_product = Exact::ZERO;
    if t.result(exact_product.checked_deposit_product(first, last))
        .is_some()
    {
        t.exact(&exact_product);
    }
    let mut exact_dot = Exact::ZERO;
    for (&a, &b) in xs.iter().zip(&reversed) {
        t.result(exact_dot.checked_deposit_product(a, b));
    }
    s.exact_dot_promoted += u64::from(!exact_dot.is_compact());
    t.exact(&exact_dot);
}

/// The whole transcript: the two vector sets, then `multisets` random multisets from `seed`.
fn transcript(seed: u64, multisets: u64) -> (String, Stats) {
    let mut t = Transcript::new();
    let mut s = Stats::default();
    rounding(&mut t, &mut s);
    conformance(&mut t, &mut s);
    let mut rng = Rng(seed);
    for round in 0..multisets {
        let xs = multiset(&mut rng, round);
        absorb(&mut t, &mut s, &mut rng, &xs);
    }
    let (hex, bytes) = t.finish();
    s.bytes = bytes;
    (hex, s)
}

#[test]
fn golden_hash_matches_the_committed_claim() {
    let (hex, s) = transcript(SEED, MULTISETS);
    assert_eq!(s.reads, 173, "{s:?}");
    assert_eq!(s.quotients, 102, "{s:?}");
    assert_eq!(s.cases, 10, "{s:?}");
    assert_eq!(s.multisets, MULTISETS, "{s:?}");
    assert!(
        s.fixed_refused > 0
            && s.fixed_bulk_refused > 0
            && s.wide_products_refused > 0
            && s.fixed_products_refused > 0
            && s.binned_products_refused > 0
            && s.exact_promoted > 0
            && s.exact_dot_promoted > 0
            && s.exact_dyadic > 0,
        "every refusal path and every heap vector are in the transcript: {s:?}"
    );
    assert_eq!(s.elements, ELEMENTS, "{s:?}");
    assert_eq!(s.bytes, BYTES, "{s:?}");
    let claim = CLAIM.trim();
    assert!(
        claim.len() == 64 && claim.bytes().all(|b| b.is_ascii_hexdigit()),
        "tests/data/golden.sha256 holds one SHA-256 in hex"
    );
    assert_eq!(
        hex, claim,
        "the transcript's hash differs from tests/data/golden.sha256. On a target other than the \
         one that blessed it, this is a finding: some op or read lands different bits there. \
         After a deliberate change to the inputs or the record layout, commit the computed hash."
    );
}

/// The hash is a function of the inputs: the same seed twice lands the same hash, and a different
/// seed a different one, so the comparison above can fail.
#[test]
fn golden_transcript_follows_its_seed() {
    let (a, sa) = transcript(1, 20);
    let (b, sb) = transcript(1, 20);
    let (c, sc) = transcript(2, 20);
    assert_eq!(sa.multisets, 20);
    assert_eq!(sa.bytes, sb.bytes);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(sa.elements, sc.elements);
}
