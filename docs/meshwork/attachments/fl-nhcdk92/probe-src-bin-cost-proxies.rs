//! The proxies beside the niche form: `W`, `Wide` as the diff (option 2); `C`, the Copy word
//! with the tag test alone (a diagnostic); `H`, the 40-byte enum of fl-t154dsp's probe.

use differential_dataflow::difference::{Abelian, IsZero, Monoid, Multiply, Semigroup};
use float_abel::{Exact, PastCap, Wide};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- the Wide newtype (option 2)

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct W(pub(crate) Wide);
impl IsZero for W {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl Semigroup for W {
    fn plus_equals(&mut self, rhs: &Self) {
        self.0 += rhs.0;
    }
}
impl Monoid for W {
    fn zero() -> Self {
        Self(Wide::default())
    }
}
impl Abelian for W {
    fn negate(&mut self) {
        self.0 = -self.0;
    }
}
impl Multiply<isize> for W {
    type Output = Self;
    fn multiply(self, rhs: &isize) -> Self {
        Self(self.0 * *rhs)
    }
}

// ------------------------------------------- the Copy word with the tag test alone (diagnostic)

/// `Exact` as it is, `Copy`, plus the niche form's tag test on every op, with the heap branch a
/// cold call: prices the test apart from everything a pointer-owning word brings.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub(crate) struct C(pub(crate) Exact);
#[cold]
#[inline(never)]
fn heap_unreachable() -> ! {
    panic!("the Copy diagnostic never holds a heap word")
}
impl C {
    #[inline]
    fn is_heap(&self) -> bool {
        // SAFETY: Exact is repr(C) with lo first.
        let lo = unsafe { *(self as *const Self).cast::<u64>() };
        lo & 1 == 0 && lo != 0
    }
}
impl IsZero for C {
    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl Semigroup for C {
    #[inline]
    fn plus_equals(&mut self, rhs: &Self) {
        if self.is_heap() || rhs.is_heap() {
            heap_unreachable();
        }
        match self.0.checked_add(&rhs.0) {
            Ok(s) => self.0 = s,
            Err(PastCap::Width) => heap_unreachable(),
            Err(cap) => panic!("{cap}"),
        }
    }
}
impl Monoid for C {
    #[inline]
    fn zero() -> Self {
        Self(Exact::ZERO)
    }
}
impl Abelian for C {
    #[inline]
    fn negate(&mut self) {
        if self.is_heap() {
            heap_unreachable();
        }
        self.0 = -self.0;
    }
}
impl Multiply<isize> for C {
    type Output = Self;
    #[inline]
    fn multiply(self, rhs: &isize) -> Self {
        if self.is_heap() {
            heap_unreachable();
        }
        match self.0.checked_mul_i64(*rhs as i64) {
            Ok(p) => Self(p),
            Err(PastCap::Width) => heap_unreachable(),
            Err(cap) => panic!("{cap}"),
        }
    }
}

// ------------------------------------------------------- the 40-byte enum proxy (fl-t154dsp's H)

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub(crate) enum H {
    C(Exact),
    W(Box<Wide>),
}
fn widen(e: &Exact) -> Wide {
    Wide::try_from(e).expect("a sum of doubles is inside Wide")
}
fn demote(w: Wide) -> H {
    match Exact::try_from(&w) {
        Ok(e) => H::C(e),
        Err(_) => H::W(Box::new(w)),
    }
}
impl IsZero for H {
    fn is_zero(&self) -> bool {
        match self {
            H::C(e) => e.is_zero(),
            H::W(w) => w.is_zero(),
        }
    }
}
impl Semigroup for H {
    fn plus_equals(&mut self, rhs: &Self) {
        let next = match (&mut *self, rhs) {
            (H::C(a), H::C(b)) => match a.checked_add(b) {
                Ok(s) => {
                    *a = s;
                    return;
                }
                Err(_) => demote(widen(a) + widen(b)),
            },
            (H::W(a), H::C(b)) => demote(**a + widen(b)),
            (H::C(a), H::W(b)) => demote(widen(a) + **b),
            (H::W(a), H::W(b)) => demote(**a + **b),
        };
        *self = next;
    }
}
impl Monoid for H {
    fn zero() -> Self {
        H::C(Exact::ZERO)
    }
}
impl Abelian for H {
    fn negate(&mut self) {
        match self {
            H::C(e) => *e = -*e,
            H::W(w) => **w = -**w,
        }
    }
}
impl Multiply<isize> for H {
    type Output = Self;
    fn multiply(self, rhs: &isize) -> Self {
        match self {
            H::C(e) => match e.checked_mul_i64(*rhs as i64) {
                Ok(s) => H::C(s),
                Err(_) => demote(widen(&e) * *rhs),
            },
            H::W(w) => demote(*w * *rhs),
        }
    }
}

