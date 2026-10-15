//! The stack-buffered working form of the heap path: a normalised sign-magnitude dyadic, and the
//! limb helpers its add and multiply use.

use std::cmp::Ordering;

// ------------------------------------------------------------ the working form for the heap path

/// Limbs a working magnitude may hold: the 2^±300 band's sums need about eleven; a product fold
/// would need more, and the real arm's cap is 544.
const CAP: usize = 64;

/// A normalised dyadic `(-1)^neg · mag · 2^exp` on the stack: `mag` odd with its top limb nonzero,
/// or the zero as `len == 0`.
pub(crate) struct Big {
    pub(crate) neg: bool,
    pub(crate) exp: i64,
    pub(crate) len: usize,
    pub(crate) mag: [u64; CAP],
}

impl Big {
    pub(crate) const ZERO: Self = Self {
        neg: false,
        exp: 0,
        len: 0,
        mag: [0; CAP],
    };

    pub(crate) fn from_compact(lo: u64, hi: u64, exp: i32) -> Self {
        let m = ((u128::from(hi) << 64) | u128::from(lo)) as i128;
        let mag = m.unsigned_abs();
        let mut b = Self::ZERO;
        b.neg = m < 0;
        b.exp = i64::from(exp);
        b.mag[0] = mag as u64;
        b.mag[1] = (mag >> 64) as u64;
        b.len = if b.mag[1] != 0 {
            2
        } else if b.mag[0] != 0 {
            1
        } else {
            0
        };
        b
    }

    pub(crate) fn from_limbs(neg: bool, exp: i32, limbs: &[u64]) -> Self {
        let mut b = Self::ZERO;
        b.neg = neg;
        b.exp = i64::from(exp);
        b.len = limbs.len();
        b.mag[..limbs.len()].copy_from_slice(limbs);
        b
    }

    pub(crate) fn limbs(&self) -> &[u64] {
        &self.mag[..self.len]
    }

    /// Bits from the lowest set bit (bit 0, since `mag` is odd) to the highest.
    pub(crate) fn bits(&self) -> u32 {
        if self.len == 0 {
            0
        } else {
            self.len as u32 * 64 - self.mag[self.len - 1].leading_zeros()
        }
    }

    /// Strips leading zero limbs, moves trailing zero bits into the exponent, and makes the zero
    /// the zero form.
    fn normalise(&mut self) {
        while self.len > 0 && self.mag[self.len - 1] == 0 {
            self.len -= 1;
        }
        if self.len == 0 {
            self.neg = false;
            self.exp = 0;
            return;
        }
        let zero_limbs = self.mag[..self.len].iter().take_while(|&&l| l == 0).count();
        if zero_limbs > 0 {
            self.mag.copy_within(zero_limbs..self.len, 0);
            self.len -= zero_limbs;
            for l in &mut self.mag[self.len..self.len + zero_limbs] {
                *l = 0;
            }
            self.exp += 64 * zero_limbs as i64;
        }
        let tz = self.mag[0].trailing_zeros();
        if tz > 0 {
            shr(&mut self.mag[..self.len], tz);
            self.exp += i64::from(tz);
            while self.len > 0 && self.mag[self.len - 1] == 0 {
                self.len -= 1;
            }
        }
    }

    /// `a + b`, aligned to the lower exponent, normalised.
    pub(crate) fn add(a: &Self, b: &Self) -> Self {
        if a.len == 0 {
            return b.copy();
        }
        if b.len == 0 {
            return a.copy();
        }
        let exp = a.exp.min(b.exp);
        let mut x = Self::ZERO;
        let mut y = Self::ZERO;
        x.len = shl_into(&mut x.mag, a.limbs(), (a.exp - exp) as u32);
        y.len = shl_into(&mut y.mag, b.limbs(), (b.exp - exp) as u32);
        let n = x.len.max(y.len) + 1;
        assert!(n <= CAP, "the probe's working form is {CAP} limbs");
        let mut r = Self::ZERO;
        r.exp = exp;
        r.len = n;
        if a.neg == b.neg {
            r.neg = a.neg;
            add_mag(&mut r.mag[..n], &x.mag[..n], &y.mag[..n]);
        } else {
            match cmp_mag(&x.mag[..n], &y.mag[..n]) {
                Ordering::Greater => {
                    r.neg = a.neg;
                    sub_mag(&mut r.mag[..n], &x.mag[..n], &y.mag[..n]);
                }
                Ordering::Less => {
                    r.neg = b.neg;
                    sub_mag(&mut r.mag[..n], &y.mag[..n], &x.mag[..n]);
                }
                Ordering::Equal => return Self::ZERO,
            }
        }
        r.normalise();
        r
    }

    pub(crate) fn mul_i64(&self, k: i64) -> Self {
        if k == 0 || self.len == 0 {
            return Self::ZERO;
        }
        let mut r = self.copy();
        r.neg ^= k < 0;
        let ku = k.unsigned_abs();
        let t = ku.trailing_zeros();
        r.exp += i64::from(t);
        let ku = ku >> t;
        let mut carry = 0u64;
        for l in &mut r.mag[..r.len] {
            let p = u128::from(*l) * u128::from(ku) + u128::from(carry);
            *l = p as u64;
            carry = (p >> 64) as u64;
        }
        if carry != 0 {
            assert!(r.len < CAP, "the probe's working form is {CAP} limbs");
            r.mag[r.len] = carry;
            r.len += 1;
        }
        r
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            neg: self.neg,
            exp: self.exp,
            len: self.len,
            mag: self.mag,
        }
    }

    pub(crate) fn neg(&self) -> Self {
        let mut r = self.copy();
        if r.len != 0 {
            r.neg = !r.neg;
        }
        r
    }

    /// Numeric order through the sign of the difference.
    pub(crate) fn cmp(&self, other: &Self) -> Ordering {
        let d = Self::add(self, &other.neg());
        if d.len == 0 {
            Ordering::Equal
        } else if d.neg {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// `dst = src << shift`, returning the limbs written.
fn shl_into(dst: &mut [u64; CAP], src: &[u64], shift: u32) -> usize {
    let limbs = (shift / 64) as usize;
    let bits = shift % 64;
    assert!(src.len() + limbs + 1 <= CAP, "the probe's working form is {CAP} limbs");
    if bits == 0 {
        dst[limbs..limbs + src.len()].copy_from_slice(src);
        return limbs + src.len();
    }
    let mut carry = 0u64;
    for (i, &s) in src.iter().enumerate() {
        dst[limbs + i] = (s << bits) | carry;
        carry = s >> (64 - bits);
    }
    dst[limbs + src.len()] = carry;
    limbs + src.len() + 1
}

/// `a >>= bits` for `0 < bits < 64`.
fn shr(a: &mut [u64], bits: u32) {
    let n = a.len();
    for i in 0..n {
        let high = if i + 1 < n { a[i + 1] << (64 - bits) } else { 0 };
        a[i] = (a[i] >> bits) | high;
    }
}

fn add_mag(r: &mut [u64], x: &[u64], y: &[u64]) {
    let mut carry = 0u64;
    for i in 0..r.len() {
        let (s1, c1) = x[i].overflowing_add(y[i]);
        let (s2, c2) = s1.overflowing_add(carry);
        r[i] = s2;
        carry = u64::from(c1 | c2);
    }
    debug_assert_eq!(carry, 0);
}

/// `r = x - y` for `x >= y`.
fn sub_mag(r: &mut [u64], x: &[u64], y: &[u64]) {
    let mut borrow = 0u64;
    for i in 0..r.len() {
        let (s1, b1) = x[i].overflowing_sub(y[i]);
        let (s2, b2) = s1.overflowing_sub(borrow);
        r[i] = s2;
        borrow = u64::from(b1 | b2);
    }
    debug_assert_eq!(borrow, 0);
}

fn cmp_mag(x: &[u64], y: &[u64]) -> Ordering {
    x.iter().rev().cmp(y.iter().rev())
}

