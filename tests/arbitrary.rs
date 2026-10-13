//! The `arbitrary` feature: `Arbitrary` on every group type, so a fuzz target draws values
//! directly. Every draw is a canonical value whose bytes round-trip, and `Exact`'s draws are
//! weighted so that a pair of them fits the width often enough to reach the operators and not
//! only the cap. Every test is named `arbitrary_*`.

use arbitrary::{Arbitrary, Unstructured};
use float_abel::{Exact, Fixed256, Wide, EXACT_MAX_EXP};

use crate::support::Rng;

/// A deterministic byte stream for `Unstructured`.
fn bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = Rng(seed);
    (0..len)
        .flat_map(|_| rng.next().to_le_bytes())
        .take(len)
        .collect()
}

#[test]
#[cfg_attr(miri, ignore = "the distribution over 4000 draws is the assertion")]
fn arbitrary_exact_draws_are_canonical_and_reach_the_operators() {
    let data = bytes(1, 1 << 20);
    let mut u = Unstructured::new(&data);
    let draws: Vec<Exact> = (0..4000)
        .map(|_| Exact::arbitrary(&mut u).unwrap())
        .collect();
    let (mut wide_mantissas, mut narrow, mut at_bound, mut specials, mut zeros, mut sums) =
        (0, 0, 0, 0, 0, 0);
    for pair in draws.chunks(2) {
        let (a, b) = (pair[0], pair[1]);
        for v in [a, b] {
            // Canonical: the bytes decode to the same word, which the decoder vouches for.
            assert_eq!(Exact::from_bytes(&v.to_bytes()), Ok(v), "{v:?}");
            let bytes = v.to_bytes();
            let hi = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
            let exp = i32::from_le_bytes(bytes[17..21].try_into().unwrap());
            wide_mantissas += usize::from(hi != 0 && hi != u64::MAX);
            narrow += usize::from((-60..60).contains(&exp));
            at_bound += usize::from(exp.abs() == EXACT_MAX_EXP);
            specials += usize::from(!v.is_finite());
            zeros += usize::from(v.is_zero());
        }
        sums += usize::from(a.checked_add(&b).is_ok());
    }
    assert!(
        wide_mantissas > 1000,
        "{wide_mantissas} draws past one limb"
    );
    assert!(narrow > 400, "{narrow} draws within 2^±60");
    assert!(at_bound > 20, "{at_bound} draws at the exponent bound");
    assert!(specials > 1000, "{specials} draws with a special count");
    assert!(zeros > 5, "{zeros} zero draws");
    // One pair in twenty or better reaches the operators; the fuzzer's coverage feedback finds
    // the rest.
    assert!(sums > 100, "{sums} of 2000 pairs fit the width");
}

#[test]
fn arbitrary_wide_and_fixed_draws_are_their_bit_patterns_and_round_trip() {
    let data = bytes(2, 1 << 16);
    let mut u = Unstructured::new(&data);
    for _ in 0..100 {
        let wide = Wide::arbitrary(&mut u).unwrap();
        assert_eq!(Wide::from_bytes(&wide.to_bytes()), Ok(wide));
        let (nan, pos, neg) = wide.specials();
        assert_eq!(Wide::from_parts(*wide.limbs(), [nan, pos, neg]), wide);
        let fixed = Fixed256::arbitrary(&mut u).unwrap();
        let bytes: [u8; Fixed256::BYTES] = fixed.to_bytes();
        assert_eq!(Fixed256::from_bytes(&bytes), Ok(fixed));
        assert_eq!(Fixed256::from_limbs(*fixed.limbs()), fixed);
    }
    // The draws differ: the stream reaches the limbs.
    let a = Wide::arbitrary(&mut u).unwrap();
    let b = Wide::arbitrary(&mut u).unwrap();
    assert_ne!(a, b);
    // An exhausted stream still draws a value.
    let mut empty = Unstructured::new(&[]);
    assert_eq!(Wide::arbitrary(&mut empty).unwrap(), Wide::ZERO);
    assert_eq!(Exact::arbitrary(&mut empty).unwrap(), Exact::ZERO);
}
