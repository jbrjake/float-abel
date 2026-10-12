//! Correct rounding, DESIGN.md § Reads and rounding: `to_f64`, `to_f32` and `div_f64` on every
//! tier against `tests/data/rounding_vectors.json`, which `scripts/gen_rounding_vectors.py`
//! derives in Python and never from this crate, and the three tiers' agreement on random
//! multisets inside `Fixed256`'s window. A vector is injected exactly: its mantissa shifted onto a
//! tier's grid and handed to `Wide::from_parts` or `Fixed256::from_limbs`, and `Exact` taken from
//! the `Wide` through the conversion, scaled by a power of two when the vector lies off `Wide`'s
//! grid. Every test is named `rounding_*` so the task verify can filter on it.

use float_abel::{Exact, Fixed256, PastCap, Wide};

use crate::support::{pow2, Rng};

/// `(-1)^negative · mant · 2^exp`, the mantissa as limbs, low first, with no zero limb on top.
struct Dyadic {
    name: String,
    tags: Vec<String>,
    negative: bool,
    mant: Vec<u64>,
    exp: i32,
}

struct Read {
    v: Dyadic,
    f64: u64,
    f64_exact: bool,
    f32: u32,
}

struct Quotient {
    v: Dyadic,
    n: u64,
    /// `None` is NaN, whatever its payload.
    f64: Option<u64>,
}

fn hex_limbs(s: &str) -> Vec<u64> {
    let digits = s.strip_prefix("0x").unwrap();
    let mut limbs = Vec::new();
    let mut end = digits.len();
    while end > 0 {
        let start = end.saturating_sub(16);
        limbs.push(u64::from_str_radix(&digits[start..end], 16).unwrap());
        end = start;
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    limbs
}

fn hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.strip_prefix("0x").unwrap(), 16).unwrap()
}

fn dyadic(v: &serde_json::Value) -> Dyadic {
    Dyadic {
        name: v["name"].as_str().unwrap().to_owned(),
        tags: v["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_owned())
            .collect(),
        negative: v["negative"].as_bool().unwrap(),
        mant: hex_limbs(v["mant"].as_str().unwrap()),
        exp: v["exp"].as_i64().unwrap() as i32,
    }
}

fn vectors() -> (Vec<Read>, Vec<Quotient>) {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("data/rounding_vectors.json")).unwrap();
    assert_eq!(v["generator"], "scripts/gen_rounding_vectors.py");
    let reads = v["reads"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| Read {
            v: dyadic(r),
            f64: hex_u64(r["f64"].as_str().unwrap()),
            f64_exact: r["f64_exact"].as_bool().unwrap(),
            f32: hex_u64(r["f32"].as_str().unwrap()) as u32,
        })
        .collect();
    let quotients = v["quotients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| Quotient {
            v: dyadic(q),
            n: q["n"].as_u64().unwrap(),
            f64: match q["f64"].as_str().unwrap() {
                "nan" => None,
                hex => Some(hex_u64(hex)),
            },
        })
        .collect();
    (reads, quotients)
}

fn tagged<'a>(vs: impl Iterator<Item = &'a Dyadic>, tag: &str) -> usize {
    vs.filter(|v| v.tags.iter().any(|t| t == tag)).count()
}

fn bit_length(mant: &[u64]) -> u64 {
    mant.last().map_or(0, |&top| {
        64 * (mant.len() as u64 - 1) + 64 - u64::from(top.leading_zeros())
    })
}

fn trailing_zeros(mant: &[u64]) -> u64 {
    mant.iter()
        .position(|&x| x != 0)
        .map_or(0, |i| 64 * i as u64 + u64::from(mant[i].trailing_zeros()))
}

/// The mantissa shifted `shift` bits up into `N` limbs, as a magnitude, or `None` when the shift is
/// negative (a set bit would fall below the grid) or the result needs more than `ceiling_bits`.
fn place<const N: usize>(mant: &[u64], shift: i64, ceiling_bits: u64) -> Option<[u64; N]> {
    let mut out = [0u64; N];
    if mant.is_empty() {
        return Some(out);
    }
    if shift < 0 || bit_length(mant) + shift as u64 > ceiling_bits {
        return None;
    }
    let (words, bits) = ((shift / 64) as usize, (shift % 64) as u32);
    for (i, &limb) in mant.iter().enumerate() {
        out[i + words] |= limb << bits;
        if bits != 0 && limb >> (64 - bits) != 0 {
            out[i + words + 1] |= limb >> (64 - bits);
        }
    }
    Some(out)
}

fn negate<const N: usize>(a: &mut [u64; N]) {
    let mut carry = 1u64;
    for x in a.iter_mut() {
        let (s, c) = (!*x).overflowing_add(carry);
        *x = s;
        carry = u64::from(c);
    }
}

fn wide_of(v: &Dyadic) -> Option<Wide> {
    let ceiling = (Wide::CEILING_EXP - Wide::LSB) as u64;
    let mut limbs = place::<{ Wide::LIMBS }>(&v.mant, i64::from(v.exp - Wide::LSB), ceiling)?;
    if v.negative {
        negate(&mut limbs);
    }
    Some(Wide::from_parts(limbs, [0; 3]))
}

fn fixed_of(v: &Dyadic) -> Option<Fixed256> {
    let ceiling = (Fixed256::CEILING_EXP - Fixed256::LSB) as u64;
    let mut limbs = place::<4>(&v.mant, i64::from(v.exp - Fixed256::LSB), ceiling)?;
    if v.negative {
        negate(&mut limbs);
    }
    Some(Fixed256::from_limbs(limbs))
}

/// `Exact` holds a vector iff the span from its lowest set bit to its highest fits 127 bits. One
/// inside `Wide`'s window comes through the conversion; one off the grid or past the ceiling is
/// built at a shifted exponent and scaled back by an exact power of two through the ring product.
fn exact_of(v: &Dyadic) -> Option<Exact> {
    if bit_length(&v.mant) - trailing_zeros(&v.mant) > 127 {
        return None;
    }
    if let Some(wide) = wide_of(v) {
        return Some(Exact::try_from(&wide).unwrap());
    }
    let shift = (-v.exp).clamp(-1023, 1074);
    let moved = Dyadic {
        name: v.name.clone(),
        tags: Vec::new(),
        negative: v.negative,
        mant: v.mant.clone(),
        exp: v.exp + shift,
    };
    let wide =
        wide_of(&moved).unwrap_or_else(|| panic!("{}: shifted by {shift} and still off", v.name));
    Some(Exact::try_from(&wide).unwrap() * Exact::from(pow2(-shift)))
}

/// A vector held exactly by one tier, with the tier's reads behind one surface.
#[derive(Clone, Copy)]
#[allow(clippy::large_enum_variant)] // Wide is 296 bytes to Exact's 32; a few hundred of them
enum Held {
    Wide(Wide),
    Fixed(Fixed256),
    Exact(Exact),
}

impl Held {
    fn name(self) -> &'static str {
        match self {
            Self::Wide(_) => "Wide",
            Self::Fixed(_) => "Fixed256",
            Self::Exact(_) => "Exact",
        }
    }

    fn to_f64(self) -> f64 {
        match self {
            Self::Wide(w) => w.to_f64(),
            Self::Fixed(f) => f.to_f64(),
            Self::Exact(e) => e.to_f64(),
        }
    }

    fn to_f32(self) -> f32 {
        match self {
            Self::Wide(w) => w.to_f32(),
            Self::Fixed(f) => f.to_f32(),
            Self::Exact(e) => e.to_f32(),
        }
    }

    fn try_to_f64(self) -> Option<f64> {
        match self {
            Self::Wide(w) => w.try_to_f64(),
            Self::Fixed(f) => f.try_to_f64(),
            Self::Exact(e) => e.try_to_f64(),
        }
    }

    fn div_f64(self, n: u64) -> f64 {
        match self {
            Self::Wide(w) => w.div_f64(n),
            Self::Fixed(f) => f.div_f64(n),
            Self::Exact(e) => e.div_f64(n),
        }
    }
}

/// Every tier that holds the vector.
fn tiers(v: &Dyadic) -> Vec<Held> {
    let out: Vec<Held> = wide_of(v)
        .map(Held::Wide)
        .into_iter()
        .chain(fixed_of(v).map(Held::Fixed))
        .chain(exact_of(v).map(Held::Exact))
        .collect();
    assert!(!out.is_empty(), "{}: no tier holds it", v.name);
    out
}

#[test]
fn rounding_vector_file_covers_every_boundary() {
    let (reads, quotients) = vectors();
    assert_eq!(reads.len(), 173);
    assert_eq!(quotients.len(), 102);
    let rs = || reads.iter().map(|r| &r.v);
    assert!(
        tagged(rs(), "tie") >= 34,
        "ties at the f64 and f32 boundaries"
    );
    assert!(
        tagged(rs(), "subnormal") >= 42,
        "subnormal results in both formats"
    );
    assert!(
        tagged(rs(), "overflow") >= 16,
        "overflow to ±∞ in both formats"
    );
    assert!(
        tagged(rs(), "sticky") >= 16,
        "sticky bits below the round bit"
    );
    assert!(tagged(rs(), "sign") >= 62, "negative twins");
    assert!(
        tagged(rs(), "zero") >= 7,
        "values that round to a signed zero"
    );
    assert!(
        tagged(rs(), "f32-differs") >= 10,
        "f32 cases where rounding via f64 first goes wrong"
    );
    assert!(tagged(rs(), "random") >= 48);
    assert_eq!(reads.iter().filter(|r| r.f64_exact).count(), 74);
    let qs = || quotients.iter().map(|q| &q.v);
    assert_eq!(tagged(qs(), "zero-divisor"), 3);
    assert!(tagged(qs(), "tie") >= 10, "quotients that land on a tie");
    assert!(tagged(qs(), "subnormal") >= 12);
    assert!(tagged(qs(), "overflow") >= 6);
    assert!(tagged(qs(), "random") >= 32);
    // Each tier holds a known share: Wide everything on its grid, Fixed256 the band above 2^-96,
    // and Exact everything of 127 bits or fewer, including the vectors Wide cannot hold.
    let held = |f: &dyn Fn(&Dyadic) -> bool| reads.iter().filter(|r| f(&r.v)).count();
    assert_eq!(held(&|v| wide_of(v).is_some()), 147);
    assert_eq!(held(&|v| fixed_of(v).is_some()), 94);
    assert_eq!(held(&|v| exact_of(v).is_some()), 165);
    assert_eq!(held(&|v| wide_of(v).is_none() && exact_of(v).is_some()), 26);
    assert_eq!(held(&|v| wide_of(v).is_some() && exact_of(v).is_none()), 8);
}

#[test]
fn rounding_to_f64_matches_cpython_on_every_tier() {
    let (reads, _) = vectors();
    assert!(reads.len() >= 170);
    let (mut checks, mut exact, mut inexact) = (0, 0, 0);
    for r in &reads {
        for held in tiers(&r.v) {
            let (tier, value) = (held.name(), held.to_f64());
            assert_eq!(value.to_bits(), r.f64, "{tier}: {}", r.v.name);
            if r.f64_exact {
                assert_eq!(
                    held.try_to_f64(),
                    Some(value),
                    "{tier}: {} is exact",
                    r.v.name
                );
                exact += 1;
            } else {
                assert_eq!(held.try_to_f64(), None, "{tier}: {} is inexact", r.v.name);
                inexact += 1;
            }
            checks += 1;
        }
    }
    assert!(
        checks >= 400 && exact >= 150 && inexact >= 200,
        "{checks} {exact} {inexact}"
    );
}

#[test]
fn rounding_to_f32_rounds_once_from_the_exact_state_on_every_tier() {
    let (reads, _) = vectors();
    assert!(reads.len() >= 170);
    let (mut checks, mut differs) = (0, 0);
    for r in &reads {
        let via_f64 = r.v.tags.iter().any(|t| t == "f32-differs");
        for held in tiers(&r.v) {
            let tier = held.name();
            assert_eq!(held.to_f32().to_bits(), r.f32, "{tier}: {}", r.v.name);
            if via_f64 {
                // The vector exists because `to_f64() as f32` is the wrong answer here.
                assert_ne!(
                    (held.to_f64() as f32).to_bits(),
                    r.f32,
                    "{tier}: {} via f64",
                    r.v.name
                );
                differs += 1;
            }
            checks += 1;
        }
    }
    assert!(checks >= 400 && differs >= 20, "{checks} {differs}");
}

#[test]
fn rounding_div_f64_matches_cpython_on_every_tier() {
    let (_, quotients) = vectors();
    assert!(quotients.len() >= 100);
    let (mut checks, mut nan) = (0, 0);
    for q in &quotients {
        for held in tiers(&q.v) {
            let (tier, got) = (held.name(), held.div_f64(q.n));
            if let Some(bits) = q.f64 {
                assert_eq!(got.to_bits(), bits, "{tier}: {}", q.v.name);
            } else {
                assert!(got.is_nan(), "{tier}: {}", q.v.name);
                nan += 1;
            }
            checks += 1;
        }
    }
    assert!(checks >= 250 && nan == 3, "{checks} {nan}");
}

/// `Exact` refuses the wide-mantissa vectors by name, and holds the ones off `Wide`'s grid.
#[test]
fn rounding_vectors_past_exacts_width_are_refused_by_name() {
    let (reads, _) = vectors();
    let mut refused = 0;
    for r in &reads {
        if let Some(wide) = wide_of(&r.v) {
            if exact_of(&r.v).is_none() {
                assert_eq!(Exact::try_from(&wide), Err(PastCap::Width), "{}", r.v.name);
                refused += 1;
            }
        }
    }
    assert!(refused >= 2, "{refused}");
}

/// Random multisets from a 60-binade band inside `Fixed256`'s window, so all three tiers hold
/// every sum: `Fixed256`, `Wide` and `Exact` must agree bit for bit on every read.
fn agree(seed: u64, multisets: usize) {
    let mut rng = Rng(seed);
    let (mut inexact, mut negative, mut f32_inexact) = (0, 0, 0);
    for _ in 0..multisets {
        let lo = -44 + (rng.next() % 80) as i32;
        let count = 1 + (rng.next() % 16) as usize;
        let (mut fixed, mut wide, mut exact) = (Fixed256::ZERO, Wide::ZERO, Exact::ZERO);
        for _ in 0..count {
            let x = rng.on_grid(lo, lo + 60);
            if rng.next() % 4 == 0 {
                fixed -= Fixed256::try_from(x).unwrap();
                wide -= Wide::from(x);
                exact -= Exact::from(x);
            } else {
                fixed += Fixed256::try_from(x).unwrap();
                wide += Wide::from(x);
                exact += Exact::from(x);
            }
        }
        let n = if rng.next() % 8 == 0 {
            rng.next()
        } else {
            1 + rng.next() % 1000
        };
        let double = fixed.to_f64().to_bits();
        assert_eq!(wide.to_f64().to_bits(), double, "{fixed:?}");
        assert_eq!(exact.to_f64().to_bits(), double, "{fixed:?}");
        let single = fixed.to_f32().to_bits();
        assert_eq!(wide.to_f32().to_bits(), single, "{fixed:?}");
        assert_eq!(exact.to_f32().to_bits(), single, "{fixed:?}");
        let tried = fixed.try_to_f64().map(f64::to_bits);
        assert_eq!(wide.try_to_f64().map(f64::to_bits), tried, "{fixed:?}");
        assert_eq!(exact.try_to_f64().map(f64::to_bits), tried, "{fixed:?}");
        let quotient = fixed.div_f64(n).to_bits();
        assert_eq!(wide.div_f64(n).to_bits(), quotient, "{fixed:?} / {n}");
        assert_eq!(exact.div_f64(n).to_bits(), quotient, "{fixed:?} / {n}");
        inexact += usize::from(tried.is_none());
        negative += usize::from(double >> 63 != 0);
        f32_inexact += usize::from(f64::from(f32::from_bits(single)).to_bits() != double);
    }
    // A sum of two or more full-fraction terms is nearly always inexact; the exact reads are the
    // single-term multisets, one in sixteen, and the rare cancellations.
    assert!(
        inexact >= multisets / 2 && multisets - inexact >= multisets / 32,
        "{inexact} inexact of {multisets}"
    );
    assert!(
        negative >= multisets / 4,
        "{negative} negative of {multisets}"
    );
    assert!(
        f32_inexact >= multisets / 2,
        "{f32_inexact} inexact in f32 of {multisets}"
    );
}

#[test]
fn rounding_tiers_agree_on_a_slice_of_random_multisets() {
    agree(0x2a9ee, 20_000);
}

/// The full million, which the push gate runs with `--include-ignored`.
#[test]
#[ignore = "ten seconds in a debug build: the push gate runs it with --include-ignored"]
fn rounding_tiers_agree_on_a_million_random_multisets() {
    agree(0x0002_a9ee_0000, 1_000_000);
}
