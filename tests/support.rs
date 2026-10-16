//! Helpers the test topics share: a deterministic random stream, a hasher, and an allocation
//! counter, so a test can assert that a refusal allocated nothing. Nothing here is a test, so the
//! file is a `mod` of tests/main.rs like a topic but carries no `#[test]`.

use core::cell::Cell;
use core::hash::{Hash, Hasher};
use std::alloc::{GlobalAlloc, Layout, System};

thread_local! {
    /// Allocations made on this thread, counted by the test binary's allocator. A `const` initialiser
    /// with no destructor, so reading it from inside `alloc` never allocates.
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

/// The system allocator with a per-thread count of its allocations, so [`allocations`] can say
/// how many a closure made with the other test threads out of the picture.
struct Counting;

// SAFETY: every method forwards to `System`, which upholds the contract; the count is the only
// addition, and reading the thread-local allocates nothing.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
        // SAFETY: the caller's contract is this method's, passed through unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
        // SAFETY: as `alloc`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
        // SAFETY: as `alloc`; `ptr` and `layout` are what this allocator handed out.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: as `realloc`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// `f`'s result and the number of allocations it made on this thread.
pub(crate) fn allocations<T>(f: impl FnOnce() -> T) -> (T, u64) {
    let before = ALLOCATIONS.with(Cell::get);
    let out = f();
    (out, ALLOCATIONS.with(Cell::get) - before)
}

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

/// `n` natively, and a hundredth of it (at least one) under miri, which interprets a limb
/// operation some thousand times slower than the machine runs it. For a sweep whose assertions
/// hold at any count; a loop whose count is the assertion keeps its literal and is ignored under
/// miri.
pub(crate) const fn rounds(n: usize) -> usize {
    if cfg!(miri) {
        let cut = n / 100;
        if cut == 0 {
            1
        } else {
            cut
        }
    } else {
        n
    }
}

/// 2^e as a double, by its bits: `powi` reciprocates, so it returns 0 for `e <= -1024` where the
/// subnormals still reach down to -1074, and miri perturbs it by a few ulps on purpose, as it
/// does every operation whose result is not specified to the bit.
pub(crate) fn pow2(e: i32) -> f64 {
    assert!((-1074..=1023).contains(&e));
    if e >= -1022 {
        f64::from_bits(((e + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (e + 1074))
    }
}

pub(crate) fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
