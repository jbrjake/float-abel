//! The laws kit run against every exact tier, and against two negative controls, a naive `f64`
//! newtype and a compact-only `Exact`, which must fail: a control that passes means the kit is
//! blind. `tests/laws_cap.rs` runs the kit at the limb cap's edge. Every test is named `laws_*`
//! so the task verify can filter on it.

use core::hash::{Hash, Hasher};
use core::ops::{Add, Mul, Neg, Sub};

use float_abel::laws::proptest::prelude::*;
use float_abel::laws::proptest::test_runner::{Config, RngSeed, TestError, TestRunner};
use float_abel::laws::{self, Case, RingCase, Weight};
use float_abel::{Exact, Fixed256, Wide};

/// One seed for every run, so a failure here is the same failure on every machine.
pub(crate) const SEED: u64 = 1;
/// DESIGN.md § Verification: 256 cases in CI with a fixed seed.
const CASES: u32 = 256;

pub(crate) fn runner() -> TestRunner {
    laws::runner(CASES, SEED)
}

/// The control's runner reports the first failing case as drawn, so every law it breaks shows at
/// once; shrinking would pare the input down to the one law that survives on two weights.
pub(crate) fn unshrunk() -> TestRunner {
    unshrunk_for(CASES)
}

pub(crate) fn unshrunk_for(cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(SEED),
        failure_persistence: None,
        max_shrink_iters: 0,
        ..Config::default()
    })
}

/// A normal double with its leading bit at `2^e` for `e` in `lo..hi`, a full fraction and either
/// sign: what `tests/support.rs` draws, as a strategy.
fn double(lo: i32, hi: i32) -> impl Strategy<Value = f64> {
    (lo..hi, any::<u64>(), any::<bool>()).prop_map(|(e, frac, neg)| {
        let x = f64::from_bits(((e + 1023) as u64) << 52 | frac >> 12);
        if neg {
            -x
        } else {
            x
        }
    })
}

fn special() -> impl Strategy<Value = f64> {
    prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY)]
}

/// `Fixed256`'s domain: on the grid, so the leading bit at or above `2^-44`, and far enough below
/// the ceiling that a fold of every weight a case draws stays inside the window.
fn fixed256() -> impl Strategy<Value = Fixed256> {
    double(-44, 88).prop_map(|x| Fixed256::try_from(x).unwrap_or_else(|e| panic!("{x:e}: {e}")))
}

/// `Wide`'s domain: any double, one in sixteen a special.
fn wide() -> impl Strategy<Value = Wide> {
    prop_oneof![1 => special(), 15 => any::<u64>().prop_map(f64::from_bits)].prop_map(Wide::from)
}

/// `Exact`'s domain for a fold: doubles whose leading bits lie within 56 binary decades of each
/// other, so a case's sum spans at most 56 + 53 + 5 of the 127 bits, with one in sixteen a special.
fn exact() -> impl Strategy<Value = Exact> {
    prop_oneof![1 => special(), 15 => double(-28, 28)].prop_map(Exact::from)
}

/// Every normal binade, `2^±300` with a full fraction and either sign, one in sixteen a special:
/// the adversary of DESIGN.md § Exactness bounds, three of whose four pairwise sums leave the
/// compact form. The closure law's strategy, since the law is only as strong as the strategy.
pub(crate) fn exact_full() -> impl Strategy<Value = Exact> {
    prop_oneof![1 => special(), 15 => double(-300, 300)].prop_map(Exact::from)
}

/// A row's product of two such doubles, deposited as `Exact::product`: a 106-bit mantissa over
/// `2^±600`, so every value drawn is a heap value.
fn exact_products() -> impl Strategy<Value = Exact> {
    (double(-300, 300), double(-300, 300)).prop_map(|(a, b)| Exact::product(a, b))
}

/// An integer multiplier as an `Exact`, for the ring on the weights above: a special amount must
/// stay an integer, so a special multiplier needs integer weights, and that is [`exact_small`].
fn exact_integer() -> impl Strategy<Value = Exact> {
    (-1024i64..1024).prop_map(|k| Exact::from(k as f64))
}

/// A ring input with the specials in: a special one time in eight, else an 11-bit integer, so
/// every product of three stays inside the width and every special amount is an integer.
fn exact_small() -> impl Strategy<Value = Exact> {
    prop_oneof![1 => special().prop_map(Exact::from), 7 => exact_integer()]
}

pub(crate) fn small_int() -> impl Strategy<Value = i64> {
    -1024i64..1024
}

#[test]
fn laws_fixed256_passes_every_check() {
    let mut r = runner();
    assert_eq!(laws::check_abelian_group(&mut r, fixed256()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, fixed256(), small_int()), Ok(()));
    assert_eq!(laws::check_canonical(&mut r, fixed256()), Ok(()));
    assert_eq!(
        laws::check_consolidation_invariance(&mut r, fixed256()),
        Ok(())
    );
}

#[test]
fn laws_wide_passes_every_check_with_specials_in_the_multiset() {
    let mut r = runner();
    assert_eq!(laws::check_abelian_group(&mut r, wide()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, wide(), small_int()), Ok(()));
    assert_eq!(laws::check_canonical(&mut r, wide()), Ok(()));
    assert_eq!(laws::check_consolidation_invariance(&mut r, wide()), Ok(()));
}

#[test]
fn laws_exact_passes_every_check_with_integer_and_exact_multipliers() {
    let mut r = runner();
    assert_eq!(laws::check_abelian_group(&mut r, exact()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, exact(), small_int()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, exact(), exact_integer()), Ok(()));
    assert_eq!(
        laws::check_ring(&mut r, exact_small(), exact_small()),
        Ok(())
    );
    assert_eq!(laws::check_canonical(&mut r, exact()), Ok(()));
    assert_eq!(
        laws::check_consolidation_invariance(&mut r, exact()),
        Ok(())
    );
}

/// The four older laws on the strategy the closure law runs on, so the kit's sweep covers the heap
/// form and not the compact word alone: a fold of 32 values over `2^±300` is a heap value through
/// most of its steps, promoted and demoted as the shuffles reorder it, the ring's special amounts
/// turn dyadic under a full-mantissa multiplier, and every law must hold across both forms.
#[test]
fn laws_exact_holds_every_law_on_every_binade() {
    let mut r = runner();
    assert_eq!(laws::check_abelian_group(&mut r, exact_full()), Ok(()));
    assert_eq!(laws::check_ring(&mut r, exact_full(), small_int()), Ok(()));
    assert_eq!(
        laws::check_ring(&mut r, exact_full(), exact_integer()),
        Ok(())
    );
    assert_eq!(laws::check_ring(&mut r, exact_full(), exact_full()), Ok(()));
    assert_eq!(
        laws::check_ring(&mut r, exact_products(), exact_integer()),
        Ok(())
    );
    assert_eq!(laws::check_canonical(&mut r, exact_full()), Ok(()));
    assert_eq!(
        laws::check_consolidation_invariance(&mut r, exact_full()),
        Ok(())
    );
}

/// The heap form holds every sum and product short of its two bounds, so closure holds where the
/// compact form alone broke it: on every binade with the specials in, by an integer and by
/// another such `Exact`, and on a multiset of row products.
#[test]
fn laws_closure_exact_holds_on_every_binade_with_specials_and_on_products() {
    let mut r = runner();
    assert_eq!(
        laws::check_closure(&mut r, exact_full(), small_int()),
        Ok(())
    );
    assert_eq!(
        laws::check_closure(&mut r, exact_full(), exact_full()),
        Ok(())
    );
    assert_eq!(
        laws::check_closure(&mut r, exact_products(), exact_integer()),
        Ok(())
    );
}

#[test]
fn laws_closure_wide_and_fixed256_hold_on_their_domains() {
    let mut r = runner();
    assert_eq!(laws::check_closure(&mut r, wide(), small_int()), Ok(()));
    assert_eq!(laws::check_closure(&mut r, fixed256(), small_int()), Ok(()));
}

/// `f64` as an engine would use it for a weight: its own sum, with structural `Eq` and `Hash`
/// through the bits.
#[derive(Clone, Copy, Debug)]
struct Naive(f64);

impl PartialEq for Naive {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Naive {}

impl Hash for Naive {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Add for Naive {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Naive(self.0 + other.0)
    }
}

impl Sub for Naive {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Naive(self.0 - other.0)
    }
}

impl Neg for Naive {
    type Output = Self;
    fn neg(self) -> Self {
        Naive(-self.0)
    }
}

impl Mul<i64> for Naive {
    type Output = Self;
    fn mul(self, k: i64) -> Self {
        Naive(self.0 * k as f64)
    }
}

impl Weight for Naive {
    fn zero() -> Self {
        Naive(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

/// Same-scale decimals, `k / 1000` for `|k| < 1000`: `0.1 + 0.2` territory.
fn naive() -> impl Strategy<Value = Naive> {
    (-999i64..1000).prop_map(|k| Naive(k as f64 / 1000.0))
}

/// The failing case the kit reported, with the laws it names.
pub(crate) fn failure<I: core::fmt::Debug>(
    outcome: Result<(), TestError<I>>,
    check: &str,
) -> (String, I) {
    match outcome {
        Err(TestError::Fail(why, input)) => (why.message().to_string(), input),
        other => panic!("the control did not fail {check}: {other:?}"),
    }
}

#[test]
fn laws_naive_f64_fails_associativity_and_inverse() {
    let (why, case) = failure(
        laws::check_abelian_group(&mut unshrunk(), naive()),
        "the group check",
    );
    assert!(case.weights.len() > 2, "{case:?}");
    assert!(why.contains("associativity"), "{why}");
    assert!(why.contains("inverse"), "{why}");
}

/// With shrinking on, the kit pares the control's failure down to two decimals.
#[test]
fn laws_naive_f64_fails_on_a_minimal_input_of_two_decimals() {
    let (why, case) = failure(
        laws::check_abelian_group(&mut runner(), naive()),
        "the group check",
    );
    assert_eq!(case.weights.len(), 2, "{case:?}");
    assert!(
        why.contains("inverse") || why.contains("associativity"),
        "{why}"
    );
}

#[test]
fn laws_naive_f64_fails_distributivity() {
    let (
        why,
        RingCase {
            weights,
            multipliers,
        },
    ) = failure(
        laws::check_ring(&mut unshrunk(), naive(), small_int()),
        "the ring check",
    );
    assert!(!weights.is_empty() && !multipliers.is_empty());
    assert!(why.contains("distributivity"), "{why}");
}

#[test]
fn laws_naive_f64_fails_canonicality() {
    let (why, case) = failure(
        laws::check_canonical(&mut unshrunk(), naive()),
        "the canonical check",
    );
    // One weight cannot fail: `a + a − a` is exact, so the witness needs two.
    assert!(case.weights.len() > 1, "{case:?}");
    assert!(why.contains("canonical"), "{why}");
}

#[test]
fn laws_naive_f64_fails_consolidation_invariance() {
    let (why, case) = failure(
        laws::check_consolidation_invariance(&mut unshrunk(), naive()),
        "the consolidation check",
    );
    // One weight cannot fail: `a + (−a)` is exact, so the witness needs two.
    assert!(case.weights.len() > 1, "{case:?}");
    assert!(why.contains("consolidation"), "{why}");
}

/// `Exact` as a build without `alloc` has it: the compact form alone, with the width as its cap.
/// A result that leaves the compact form is refused by panic, as that build's operators refuse
/// it, so the kit must see the cap on a strategy that reaches it, and cannot on one that does not.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Compact(Exact);

impl Compact {
    fn fitting(value: Exact, what: &str) -> Self {
        assert!(
            value.is_compact(),
            "Compact: {what} spans more than the 127-bit width"
        );
        Compact(value)
    }
}

impl Add for Compact {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::fitting(self.0 + other.0, "a sum")
    }
}

impl Sub for Compact {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::fitting(self.0 - other.0, "a difference")
    }
}

impl Neg for Compact {
    type Output = Self;
    fn neg(self) -> Self {
        Compact(-self.0)
    }
}

impl Mul<i64> for Compact {
    type Output = Self;
    fn mul(self, k: i64) -> Self {
        Self::fitting(self.0 * k, "a product")
    }
}

impl Weight for Compact {
    fn zero() -> Self {
        Compact(Exact::ZERO)
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

fn compact_full() -> impl Strategy<Value = Compact> {
    exact_full().prop_map(Compact)
}

/// The law is only as strong as the strategy: inside 56 binades the capped type passes, as the
/// compact `Exact` passed the kit before the heap arm.
#[test]
fn laws_closure_compact_control_holds_inside_its_width() {
    assert_eq!(
        laws::check_closure(&mut runner(), exact().prop_map(Compact), small_int()),
        Ok(())
    );
}

#[test]
fn laws_closure_compact_control_fails_past_its_width() {
    let (why, case) = failure(
        laws::check_closure(&mut unshrunk(), compact_full(), small_int()),
        "the closure check",
    );
    // One weight cannot fail: its sum with itself, its product by a small integer and its fold
    // all stay inside 64 bits.
    assert!(case.weights.len() > 1, "{case:?}");
    assert!(why.contains("closure"), "{why}");
    assert!(why.contains("127-bit width"), "{why}");
}

/// With shrinking on, the kit pares the control's failure down to two values whose sum or
/// difference leaves the compact form.
#[test]
fn laws_closure_compact_control_fails_on_a_minimal_input_of_two_values() {
    let (why, case) = failure(
        laws::check_closure(&mut runner(), compact_full(), small_int()),
        "the closure check",
    );
    assert_eq!(case.weights.len(), 2, "{case:?}");
    let (a, b) = (&case.weights[0].0, &case.weights[1].0);
    assert!(!(a + b).is_compact() || !(a - b).is_compact(), "{case:?}");
    assert!(why.contains("closure"), "{why}");
}

/// The seed is honoured: one seed is one run, and another seed is another stream.
#[test]
fn laws_runner_is_deterministic_in_its_seed() {
    let first: Result<(), TestError<Case<Naive>>> =
        laws::check_abelian_group(&mut laws::runner(CASES, 7), naive());
    let again = laws::check_abelian_group(&mut laws::runner(CASES, 7), naive());
    assert!(first.is_err());
    assert_eq!(first, again);
    assert_eq!(
        laws::runner(1, 7).rng().next_u64(),
        laws::runner(1, 7).rng().next_u64()
    );
    assert_ne!(
        laws::runner(1, 7).rng().next_u64(),
        laws::runner(1, 8).rng().next_u64()
    );
}
