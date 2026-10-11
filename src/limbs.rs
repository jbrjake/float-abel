//! Two's-complement arithmetic on little-endian `u64` limb slices, shared by every fixed-width
//! tier. Nothing here executes a floating-point instruction.

use core::cmp::Ordering;

/// The sign bit of a two's-complement value.
pub(crate) fn is_negative(a: &[u64]) -> bool {
    a.last().is_some_and(|&top| top >> 63 != 0)
}

pub(crate) fn is_zero(a: &[u64]) -> bool {
    a.iter().all(|&x| x == 0)
}

/// `a += b`, wrapping. Returns `true` on signed overflow: the operands agreed on a sign and the
/// result disagrees with it.
pub(crate) fn add_assign(a: &mut [u64], b: &[u64]) -> bool {
    debug_assert_eq!(a.len(), b.len());
    let (sa, sb) = (is_negative(a), is_negative(b));
    let mut carry = 0u64;
    for (x, &y) in a.iter_mut().zip(b) {
        let (s1, c1) = x.overflowing_add(y);
        let (s2, c2) = s1.overflowing_add(carry);
        *x = s2;
        // At most one of the two additions can carry: y + carry ≤ 2^64 - 1 + 1.
        carry = u64::from(c1 | c2);
    }
    sa == sb && is_negative(a) != sa
}

/// `a = -a`, wrapping. Returns `true` on overflow, which happens for the most negative value
/// alone; its bit pattern is then unchanged, and reads as the right magnitude unsigned.
pub(crate) fn negate(a: &mut [u64]) -> bool {
    let was_negative = is_negative(a);
    let mut carry = 1u64;
    for x in a.iter_mut() {
        let (s, c) = (!*x).overflowing_add(carry);
        *x = s;
        carry = u64::from(c);
    }
    was_negative && is_negative(a)
}

/// Signed comparison of two values of the same width.
pub(crate) fn cmp(a: &[u64], b: &[u64]) -> Ordering {
    debug_assert_eq!(a.len(), b.len());
    let Some((&ta, la)) = a.split_last() else {
        return Ordering::Equal;
    };
    let Some((&tb, lb)) = b.split_last() else {
        return Ordering::Equal;
    };
    match (ta as i64).cmp(&(tb as i64)) {
        Ordering::Equal => la.iter().rev().cmp(lb.iter().rev()),
        unequal => unequal,
    }
}

/// `a *= m` on an unsigned magnitude. Returns the limb carried out of the top.
pub(crate) fn mul_assign_u64(a: &mut [u64], m: u64) -> u64 {
    let mut carry = 0u128;
    for x in a.iter_mut() {
        let p = u128::from(*x) * u128::from(m) + carry;
        *x = p as u64;
        carry = p >> 64;
    }
    carry as u64
}

/// Copies `src` into `dst` as a magnitude. Returns whether `src` was negative.
pub(crate) fn abs_into(src: &[u64], dst: &mut [u64]) -> bool {
    dst.copy_from_slice(src);
    if is_negative(src) {
        negate(dst);
        true
    } else {
        false
    }
}

/// `dst = mag << shift`. The caller has checked that the shifted value fits, so a nonzero word
/// past the end is a bug and panics as an index out of range.
pub(crate) fn set_shifted(dst: &mut [u64], mag: u128, shift: u32) {
    dst.fill(0);
    let limb = (shift / 64) as usize;
    let off = shift % 64;
    let (lo, hi) = (mag as u64, (mag >> 64) as u64);
    let words = if off == 0 {
        [lo, hi, 0]
    } else {
        [
            lo << off,
            (lo >> (64 - off)) | (hi << off),
            hi >> (64 - off),
        ]
    };
    for (i, &w) in words.iter().enumerate() {
        if w != 0 {
            dst[limb + i] = w;
        }
    }
}

/// The index of the highest set bit, with bit 0 the lowest bit of limb 0, or `None` for zero.
pub(crate) fn top_bit(a: &[u64]) -> Option<u64> {
    a.iter()
        .rposition(|&x| x != 0)
        .map(|i| 64 * i as u64 + u64::from(a[i].ilog2()))
}

/// Bits `[lo, lo + n)` as an integer, for `n ≤ 64`; bits past the top read as zero.
pub(crate) fn window(a: &[u64], lo: u64, n: u32) -> u64 {
    debug_assert!(n <= 64);
    let limb = (lo / 64) as usize;
    let off = (lo % 64) as u32;
    let at = |i: usize| a.get(i).copied().unwrap_or(0);
    let raw = if off == 0 {
        at(limb)
    } else {
        (at(limb) >> off) | (at(limb + 1) << (64 - off))
    };
    if n == 64 {
        raw
    } else {
        raw & ((1u64 << n) - 1)
    }
}

/// Whether any bit strictly below `lo` is set.
pub(crate) fn any_below(a: &[u64], lo: u64) -> bool {
    let limb = (lo / 64) as usize;
    let off = (lo % 64) as u32;
    let whole = a.iter().take(limb).any(|&x| x != 0);
    let part = off != 0 && a.get(limb).is_some_and(|&x| x & ((1u64 << off) - 1) != 0);
    whole || part
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_i128(v: i128) -> [u64; 2] {
        [v as u64, (v >> 64) as u64]
    }

    #[test]
    fn add_matches_i128_and_flags_signed_overflow() {
        let cases: [(i128, i128); 6] = [
            (1, 2),
            (-1, 1),
            (i128::MAX, -1),
            (i128::MIN, 1),
            (1 << 64, 1 << 64),
            (-(1 << 100), -(1 << 100)),
        ];
        for (x, y) in cases {
            let mut a = from_i128(x);
            let overflow = add_assign(&mut a, &from_i128(y));
            assert!(!overflow, "{x} + {y}");
            assert_eq!(a, from_i128(x + y), "{x} + {y}");
        }
        let mut a = from_i128(i128::MAX);
        assert!(add_assign(&mut a, &from_i128(1)));
        let mut a = from_i128(i128::MIN);
        assert!(add_assign(&mut a, &from_i128(-1)));
    }

    #[test]
    fn negate_matches_i128_except_at_min() {
        for x in [0i128, 1, -1, 1 << 64, -(1 << 64), i128::MAX, i128::MIN + 1] {
            let mut a = from_i128(x);
            assert!(!negate(&mut a), "{x}");
            assert_eq!(a, from_i128(-x), "{x}");
        }
        let mut a = from_i128(i128::MIN);
        assert!(negate(&mut a));
        assert_eq!(a, from_i128(i128::MIN));
        assert_eq!(a, [0, 1 << 63], "reads as 2^127 unsigned");
    }

    #[test]
    fn cmp_is_signed() {
        let vals = [i128::MIN, -(1 << 70), -1, 0, 1, 1 << 70, i128::MAX];
        for &x in &vals {
            for &y in &vals {
                assert_eq!(cmp(&from_i128(x), &from_i128(y)), x.cmp(&y), "{x} vs {y}");
            }
        }
    }

    #[test]
    fn mul_by_u64_matches_u128() {
        let mut a = [u64::MAX, 0];
        assert_eq!(mul_assign_u64(&mut a, 3), 0);
        assert_eq!(a, from_i128(3 * i128::from(u64::MAX)));
        let mut b = [u64::MAX, u64::MAX];
        assert_eq!(mul_assign_u64(&mut b, 2), 1);
        assert_eq!(b, [u64::MAX - 1, u64::MAX]);
    }

    #[test]
    fn set_shifted_places_across_limb_boundaries() {
        let mut d = [0u64; 3];
        set_shifted(&mut d, 1, 0);
        assert_eq!(d, [1, 0, 0]);
        set_shifted(&mut d, 1, 64);
        assert_eq!(d, [0, 1, 0]);
        set_shifted(&mut d, 0b11, 63);
        assert_eq!(d, [1 << 63, 1, 0]);
        set_shifted(&mut d, u128::MAX, 1);
        assert_eq!(d, [u64::MAX - 1, u64::MAX, 1]);
        set_shifted(&mut d, 5, 130);
        assert_eq!(d, [0, 0, 5 << 2]);
    }

    #[test]
    fn top_bit_window_and_any_below() {
        assert_eq!(top_bit(&[0, 0]), None);
        assert_eq!(top_bit(&[1, 0]), Some(0));
        assert_eq!(top_bit(&[0, 1 << 5]), Some(69));
        let a = [0xFFFF_FFFF_FFFF_FFFF, 0x1234];
        assert_eq!(window(&a, 0, 64), u64::MAX);
        assert_eq!(window(&a, 60, 8), 0b0100_1111);
        assert_eq!(window(&a, 64, 16), 0x1234);
        assert_eq!(window(&a, 120, 64), 0);
        assert!(!any_below(&a, 0));
        assert!(any_below(&a, 1));
        assert!(any_below(&a, 64));
        assert!(!any_below(&[0, 0x1234], 66));
        assert!(any_below(&[0, 0x1234], 67));
    }
}
