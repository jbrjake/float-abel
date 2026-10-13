//! The plain-word contract an engine's cast over a mapping relies on: `repr(C)`, no padding, no
//! invalid bit pattern, and a word alignment. Size and alignment are pinned by `const` assertions,
//! which fail the build rather than a test, so the tests repeat them where the `layout_` filter
//! can report. The bytes of a value are read through `to_le_bytes` of each limb and counter, never
//! through `unsafe`, and a value rebuilt from them through the safe constructors equals the
//! original structurally: every bit pattern is a value, and a value is nothing but its bits.

use core::mem::{align_of, size_of};
use float_abel::{Exact, Fixed256, Wide};

use crate::support::pow2;

const WIDE_BYTES: usize = 34 * 8 + 3 * 8;
const FIXED256_BYTES: usize = 4 * 8;

const _: () = assert!(size_of::<Wide>() == WIDE_BYTES && align_of::<Wide>() == 8);
const _: () = assert!(size_of::<Fixed256>() == FIXED256_BYTES && align_of::<Fixed256>() == 8);
const _: () = assert!(size_of::<Exact>() == 32 && align_of::<Exact>() == 8);

#[test]
fn layout_sizes_are_the_field_sums_and_alignment_is_one_word() {
    // A size equal to the sum of the fields is the absence of padding.
    assert_eq!(size_of::<Wide>(), WIDE_BYTES);
    assert_eq!(size_of::<Fixed256>(), FIXED256_BYTES);
    assert_eq!(size_of::<Exact>(), 32);
    assert_eq!(align_of::<Wide>(), 8);
    assert_eq!(align_of::<Fixed256>(), 8);
    assert_eq!(align_of::<Exact>(), 8);
}

/// The wire body of a `Wide`: the limbs least significant first, then the `[n, p, q]` counts,
/// each little-endian.
fn wide_bytes(w: &Wide) -> Vec<u8> {
    let mut out = Vec::with_capacity(WIDE_BYTES);
    for limb in w.limbs() {
        out.extend_from_slice(&limb.to_le_bytes());
    }
    let (n, p, q) = w.specials();
    for count in [n, p, q] {
        out.extend_from_slice(&count.to_le_bytes());
    }
    out
}

fn wide_from_bytes(bytes: &[u8]) -> Wide {
    assert_eq!(bytes.len(), WIDE_BYTES);
    let mut limbs = [0u64; 34];
    for (limb, chunk) in limbs.iter_mut().zip(bytes[..34 * 8].chunks_exact(8)) {
        *limb = u64::from_le_bytes(chunk.try_into().unwrap());
    }
    let mut specials = [0i64; 3];
    for (count, chunk) in specials.iter_mut().zip(bytes[34 * 8..].chunks_exact(8)) {
        *count = i64::from_le_bytes(chunk.try_into().unwrap());
    }
    Wide::from_parts(limbs, specials)
}

fn fixed_bytes(f: &Fixed256) -> Vec<u8> {
    f.limbs().iter().flat_map(|l| l.to_le_bytes()).collect()
}

fn fixed_from_bytes(bytes: &[u8]) -> Fixed256 {
    assert_eq!(bytes.len(), FIXED256_BYTES);
    let mut limbs = [0u64; 4];
    for (limb, chunk) in limbs.iter_mut().zip(bytes.chunks_exact(8)) {
        *limb = u64::from_le_bytes(chunk.try_into().unwrap());
    }
    Fixed256::from_limbs(limbs)
}

#[test]
fn layout_wide_bytes_are_the_limbs_then_the_counters_and_rebuild_the_value() {
    let mut w = Wide::ZERO;
    for x in [
        1.5,
        -2.0e300,
        5e-324,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
    ] {
        w.deposit(x);
    }
    let bytes = wide_bytes(&w);
    assert_eq!(bytes.len(), size_of::<Wide>());
    // The finite part is negative, so the top limb is all ones and the rebuild is not the identity
    // on zero; the counters sit after the 272 limb bytes.
    assert_eq!(&bytes[33 * 8..34 * 8], &[0xff; 8]);
    assert_eq!(&bytes[34 * 8..35 * 8], &1i64.to_le_bytes());
    assert_eq!(&bytes[35 * 8..36 * 8], &2i64.to_le_bytes());
    assert_eq!(&bytes[36 * 8..], &1i64.to_le_bytes());
    let back = wide_from_bytes(&bytes);
    assert_eq!(back, w);
    assert_eq!(back.limbs(), w.limbs());
    assert_eq!(back.specials(), (1, 2, 1));
    assert_eq!(back.to_f64().to_bits(), w.to_f64().to_bits());
    // A different value has different bytes.
    assert_ne!(wide_bytes(&(w + Wide::from(1.0))), bytes);
}

#[test]
fn layout_fixed256_bytes_are_its_limbs_and_rebuild_the_value() {
    let mut f = Fixed256::ZERO;
    for x in [-1.5, pow2(90), pow2(-44)] {
        f.try_deposit(x).unwrap();
    }
    let bytes = fixed_bytes(&f);
    assert_eq!(bytes.len(), size_of::<Fixed256>());
    // 2^-44 is one unit above the floor of the grid, so bit 52 of limb 0 is set.
    assert_eq!(bytes[6] & 0x10, 0x10);
    let back = fixed_from_bytes(&bytes);
    assert_eq!(back, f);
    assert_eq!(back.limbs(), f.limbs());
    assert_eq!(back.to_f64().to_bits(), f.to_f64().to_bits());
    assert_ne!(fixed_bytes(&-f), bytes);
    assert_eq!(fixed_from_bytes(&fixed_bytes(&-f)), -f);
}

/// splitmix64: a deterministic stream of arbitrary bit patterns.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[test]
fn layout_every_bit_pattern_is_a_value_that_reads_back_as_itself() {
    let mut state = 0x5eed;
    for _ in 0..64 {
        let mut limbs = [0u64; 34];
        for l in &mut limbs {
            *l = splitmix(&mut state);
        }
        let specials = [
            splitmix(&mut state) as i64,
            splitmix(&mut state) as i64,
            splitmix(&mut state) as i64,
        ];
        let w = Wide::from_parts(limbs, specials);
        assert_eq!(w.limbs(), &limbs);
        assert_eq!(w.specials(), (specials[0], specials[1], specials[2]));
        assert_eq!(wide_from_bytes(&wide_bytes(&w)), w);

        let mut fl = [0u64; 4];
        for l in &mut fl {
            *l = splitmix(&mut state);
        }
        let f = Fixed256::from_limbs(fl);
        assert_eq!(f.limbs(), &fl);
        assert_eq!(fixed_from_bytes(&fixed_bytes(&f)), f);
    }
}
