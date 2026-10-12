#!/usr/bin/env python3
"""Writes tests/data/rounding_vectors.json, the oracle for `to_f64`, `to_f32` and `div_f64`.

Every vector is an exact dyadic `(-1)^negative · mant · 2^exp` with the bits it must round to, and
a quotient vector adds the divisor `n`. The expected bits come from CPython, never from this
crate:

- `f64` is `float(Fraction)`: `int / int` true division, which CPython rounds correctly, with
  `OverflowError` read as `±inf`.
- `f32` goes through `struct.pack('<f', d)`, the C `(float)` cast of a double, which rounds once
  provided `d` is the Fraction rounded *to odd* into 53 bits. An odd-rounded intermediate with two
  or more spare bits makes the second nearest-even rounding land on the correctly rounded result
  (Boldo and Melquiond, "Emulation of a FMA and correctly rounded sums", 2008), where
  `struct.pack('<f', float(frac))` would double-round at every `f32` tie. Overflow is read as `±inf`
  and anything at or past 2^128 is `±inf` without a pack.

A third derivation, a plain integer rounder over the Fraction, must agree with both for every
vector, or the generator dies instead of writing the file. Run it as
`python3 -I scripts/gen_rounding_vectors.py`; the output is deterministic.
"""

import json
import math
import struct
import sys
from fractions import Fraction
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "tests" / "data" / "rounding_vectors.json"

F64 = (53, 11)
F32 = (24, 8)


def floor_log2(a: Fraction) -> int:
    """e with 2^e <= a < 2^(e+1), for a > 0."""
    num, den = a.numerator, a.denominator
    e = num.bit_length() - den.bit_length()
    if (num << max(0, -e)) < (den << max(0, e)):
        e -= 1
    assert Fraction(2) ** e <= a < Fraction(2) ** (e + 1)
    return e


def round_bits(negative: bool, a: Fraction, fmt) -> int:
    """The bit pattern of `(-1)^negative · a` rounded to nearest-even in the format, in integer
    arithmetic: the third derivation, checked against CPython for every vector."""
    precision, exp_bits = fmt
    frac_bits = precision - 1
    bias = (1 << (exp_bits - 1)) - 1
    emax, emin = bias, 1 - bias
    min_weight = emin - frac_bits
    sign = int(negative) << (exp_bits + frac_bits)
    inf = ((1 << exp_bits) - 1) << frac_bits
    if a == 0:
        return sign
    e = floor_log2(a)
    lsb = max(e - frac_bits, min_weight)
    num, den = a.numerator, a.denominator
    q, r = divmod(num << max(0, -lsb), den << max(0, lsb))
    scaled_den = den << max(0, lsb)
    if 2 * r > scaled_den or (2 * r == scaled_den and q & 1):
        q += 1
    if q == 0:
        return sign
    bits = q.bit_length()
    top = lsb + bits - 1
    if top > emax:
        return sign | inf
    if top >= emin:
        sig = q >> (bits - precision) if bits > precision else q << (precision - bits)
        return sign | ((top + bias) << frac_bits) | (sig & ((1 << frac_bits) - 1))
    return sign | (q << (lsb - min_weight))


def f64_bits_cpython(negative: bool, a: Fraction) -> int:
    try:
        d = float(a)
    except OverflowError:
        d = math.inf
    if negative:
        d = -d
    return struct.unpack("<Q", struct.pack("<d", d))[0]


def to_double_round_to_odd(a: Fraction) -> float:
    """`a` rounded to odd into binary64: truncated to 53 bits (or to the subnormal floor), with the
    lowest bit forced to one whenever anything was cut. Exact as a Python float."""
    if a == 0:
        return 0.0
    e = floor_log2(a)
    lsb = max(e - 52, -1074)
    num, den = a.numerator, a.denominator
    q, r = divmod(num << max(0, -lsb), den << max(0, lsb))
    if r:
        q |= 1
    return math.ldexp(q, lsb)


def f32_bits_cpython(negative: bool, a: Fraction) -> int:
    if a != 0 and floor_log2(a) >= 128:
        return (int(negative) << 31) | 0x7F800000
    d = to_double_round_to_odd(a)
    if negative:
        d = -d
    try:
        packed = struct.pack("<f", d)
    except OverflowError:
        return (int(negative) << 31) | 0x7F800000
    return struct.unpack("<I", packed)[0]


def f32_bits_via_f64(negative: bool, a: Fraction) -> int:
    """What rounding through `f64` first would give: the wrong answer the `f32-differs` vectors
    exist to catch. Not an oracle."""
    d = struct.unpack("<d", struct.pack("<Q", f64_bits_cpython(negative, a)))[0]
    if math.isinf(d):
        return (int(negative) << 31) | 0x7F800000
    try:
        return struct.unpack("<I", struct.pack("<f", d))[0]
    except OverflowError:
        return (int(negative) << 31) | 0x7F800000


reads = []
quotients = []


def read(name: str, tags, negative: bool, mant: int, exp: int):
    assert mant >= 0
    a = Fraction(mant) * Fraction(2) ** exp
    f64 = f64_bits_cpython(negative, a)
    f32 = f32_bits_cpython(negative, a)
    assert f64 == round_bits(negative, a, F64), f"{name}: f64 derivations disagree"
    assert f32 == round_bits(negative, a, F32), f"{name}: f32 derivations disagree"
    differs = f32 != f32_bits_via_f64(negative, a)
    assert differs == ("f32-differs" in tags), f"{name}: f32 via f64 {'differs' if differs else 'agrees'}"
    # Whether the double is the value itself, which `try_to_f64` must report as `Some`.
    d = struct.unpack("<d", struct.pack("<Q", f64))[0]
    f64_exact = not math.isinf(d) and Fraction(d) == (-a if negative else a)
    assert f64_exact == ("exact" in tags) or "random" in tags, f"{name}: exactness mislabeled"
    reads.append(
        {
            "name": name,
            "tags": sorted(tags),
            "negative": negative,
            "mant": hex(mant),
            "exp": exp,
            "f64": f"{f64:#018x}",
            "f64_exact": f64_exact,
            "f32": f"{f32:#010x}",
        }
    )


def quotient(name: str, tags, negative: bool, mant: int, exp: int, n: int):
    assert mant >= 0 and 0 <= n < 1 << 64
    if n == 0:
        # IEEE: 0/0 is NaN, x/0 is ±∞. Any NaN bit pattern is accepted by the test.
        f64 = "nan" if mant == 0 else f"{(int(negative) << 63) | 0x7FF0000000000000:#018x}"
    else:
        a = Fraction(mant) * Fraction(2) ** exp / n
        bits = f64_bits_cpython(negative, a)
        assert bits == round_bits(negative, a, F64), f"{name}: f64 derivations disagree"
        f64 = f"{bits:#018x}"
    quotients.append(
        {
            "name": name,
            "tags": sorted(tags),
            "negative": negative,
            "mant": hex(mant),
            "exp": exp,
            "n": n,
            "f64": f64,
        }
    )


def both_signs(name: str, tags, mant: int, exp: int):
    read(name, tags, False, mant, exp)
    read(f"{name}, negative", tags + ["sign"], True, mant, exp)


# --- Exact values, which every tier reads back bit for bit ------------------------------------

read("zero", ["exact", "zero"], False, 0, 0)
both_signs("one", ["exact"], 1, 0)
both_signs("three halves", ["exact"], 3, -1)
both_signs("2^53 - 1, the last odd double", ["exact"], (1 << 53) - 1, 0)
both_signs("2^64", ["exact"], 1, 64)
both_signs("2^-1074, the smallest subnormal", ["exact", "subnormal"], 1, -1074)
both_signs("2^-1022, the smallest normal", ["exact"], 1, -1022)
both_signs("the largest subnormal", ["exact", "subnormal"], (1 << 52) - 1, -1074)
both_signs("f64::MAX", ["exact"], (1 << 53) - 1, 971)
both_signs("an even mantissa with trailing zeros", ["exact"], 5 << 40, -30)
both_signs("2^24 + 2, exact in both formats", ["exact"], (1 << 24) + 2, 0)

# --- Ties at the f64 boundary -----------------------------------------------------------------

both_signs("2^53 + 1 ties to even, down", ["tie"], (1 << 53) + 1, 0)
both_signs("2^53 + 3 ties to even, up", ["tie"], (1 << 53) + 3, 0)
both_signs("2^53 + 5 ties to even, down", ["tie"], (1 << 53) + 5, 0)
both_signs("2^54 - 1 ties up into the next binade", ["tie"], (1 << 54) - 1, 0)
both_signs("2^53 + 1 with a sticky bit far below is above the tie", ["sticky"], ((1 << 53) + 1 << 64) + 1, -64)
both_signs("2^53 + 1 with a sticky bit 130 bits below, across limbs", ["sticky"], ((1 << 53) + 1 << 130) + 1, -96)
both_signs("2^53 + 3 with a sticky bit below is still up", ["sticky"], ((1 << 53) + 3 << 70) + 1, -70)
both_signs("just below the 2^53 + 1 tie rounds down", ["nearest"], ((1 << 53) + 1 << 70) - 1, -70)
both_signs("just below the 2^53 + 3 tie rounds down to 2^53 + 2", ["nearest"], ((1 << 53) + 3 << 70) - 1, -70)
both_signs("2^53 + 2 + a quarter rounds down", ["nearest"], ((1 << 53) + 2 << 2) + 1, -2)
both_signs("2^53 + 2 + three quarters rounds up", ["nearest"], ((1 << 53) + 2 << 2) + 3, -2)
both_signs("a 100-bit mantissa across a limb boundary", ["nearest"], (1 << 100) + (1 << 47) + 1, -60)
both_signs("a 100-bit mantissa across a limb boundary, tie up", ["tie"], (1 << 100) + (3 << 47), -60)
both_signs("a 100-bit mantissa across a limb boundary, tie down", ["tie"], (1 << 100) + (1 << 47), -60)

# --- The f64 subnormal floor ------------------------------------------------------------------

both_signs("2^-1075 ties to zero", ["tie", "subnormal", "zero"], 1, -1075)
both_signs("2^-1075 + 2^-1200 rounds up to 2^-1074", ["sticky", "subnormal"], (1 << 125) + 1, -1200)
both_signs("3 · 2^-1075 ties up to 2^-1073", ["tie", "subnormal"], 3, -1075)
both_signs("3 · 2^-1076 rounds to 2^-1074", ["nearest", "subnormal"], 3, -1076)
both_signs("5 · 2^-1076 rounds to 2^-1074", ["nearest", "subnormal"], 5, -1076)
both_signs("7 · 2^-1076 rounds to 2^-1073", ["nearest", "subnormal"], 7, -1076)
both_signs("2^-1076 rounds to zero", ["nearest", "subnormal", "zero"], 1, -1076)
both_signs("a subnormal with 60 bits rounds at the floor, not the precision", ["nearest", "subnormal"], (1 << 60) + (1 << 20) - 1, -1120)
both_signs("halfway to the smallest normal ties up to it", ["tie", "subnormal"], (1 << 53) - 1, -1075)
both_signs("just under halfway to the smallest normal stays subnormal", ["nearest", "subnormal"], ((1 << 53) - 1 << 10) - 1, -1085)
both_signs("the largest subnormal plus a quarter unit stays", ["nearest", "subnormal"], ((1 << 52) - 1 << 2) + 1, -1076)

# --- Overflow to ±∞ ---------------------------------------------------------------------------

both_signs("f64::MAX plus half an ulp ties to even, which is infinity", ["tie", "overflow"], (1 << 54) - 1, 970)
both_signs("f64::MAX plus just under half an ulp stays finite", ["nearest"], ((1 << 54) - 1 << 20) - 1, 950)
both_signs("f64::MAX plus three quarters of an ulp overflows", ["nearest", "overflow"], (1 << 55) - 1, 969)
both_signs("2^1024", ["overflow"], 1, 1024)
both_signs("2^1100, past Wide's ceiling", ["overflow"], 1, 1100)
both_signs("2^1037, Wide's ceiling", ["overflow"], 1, 1037)

# --- The f32 boundary: ties, the floor, overflow, and where f64 first goes wrong --------------

both_signs("2^24 + 1 ties to even in f32, exact in f64", ["exact", "tie"], (1 << 24) + 1, 0)
both_signs("2^24 + 3 ties up in f32", ["exact", "tie"], (1 << 24) + 3, 0)
both_signs("2^25 - 1 ties up into the next f32 binade", ["exact", "tie"], (1 << 25) - 1, 0)
both_signs("2^24 + 1 with a sticky bit below rounds up in f32", ["exact", "sticky"], ((1 << 24) + 1 << 20) + 1, -20)
both_signs("2^-150 ties to zero in f32", ["exact", "tie", "subnormal", "zero"], 1, -150)
both_signs("2^-150 + 2^-200 rounds up to 2^-149 in f32", ["exact", "sticky", "subnormal"], (1 << 50) + 1, -200)
both_signs("3 · 2^-150 ties up to 2^-148 in f32", ["exact", "tie", "subnormal"], 3, -150)
both_signs("2^-149, the smallest f32 subnormal", ["exact", "subnormal"], 1, -149)
both_signs("2^-126, the smallest f32 normal", ["exact"], 1, -126)
both_signs("halfway to the smallest f32 normal ties up to it", ["exact", "tie", "subnormal"], (1 << 24) - 1, -150)
both_signs("just under halfway to the smallest f32 normal stays subnormal", ["exact", "nearest", "subnormal"], ((1 << 24) - 1 << 10) - 1, -160)
both_signs("f32::MAX", ["exact"], (1 << 24) - 1, 104)
both_signs("f32::MAX plus half an ulp ties to even, which is infinity", ["exact", "tie", "overflow"], (1 << 25) - 1, 103)
both_signs("f32::MAX plus just under half an ulp stays finite", ["exact", "nearest"], ((1 << 25) - 1 << 20) - 1, 83)
both_signs("2^128 overflows f32", ["exact", "overflow"], 1, 128)
both_signs("2^200 overflows f32 and is exact in f64", ["exact", "overflow"], 1, 200)
both_signs(
    "just above the 2^24 + 1 tie: f64 lands on the tie and f32 via f64 goes down",
    ["f32-differs", "sticky"],
    ((1 << 24) + 1 << 30) + 1,
    -30,
)
both_signs(
    "just below the 2^24 + 3 tie: f64 lands on the tie and f32 via f64 goes up",
    ["f32-differs", "nearest"],
    ((1 << 24) + 3 << 30) - 1,
    -30,
)
both_signs(
    "just above 2^-150: f64 lands on the f32 tie and f32 via f64 is zero",
    ["f32-differs", "sticky", "subnormal"],
    (1 << 60) + 1,
    -210,
)
both_signs(
    "just below f32::MAX plus half an ulp: f64 lands on the tie and f32 via f64 overflows",
    ["f32-differs", "nearest"],
    ((1 << 25) - 1 << 60) - 1,
    43,
)
both_signs(
    "just below the halfway point to 2^-126: f64 lands on the tie and f32 via f64 goes normal",
    ["f32-differs", "nearest", "subnormal"],
    ((1 << 24) - 1 << 60) - 1,
    -210,
)

# --- Quotients: div_f64 against float(Fraction(mant · 2^exp, n)) ------------------------------


def both_sign_quotients(name: str, tags, mant: int, exp: int, n: int):
    quotient(name, tags, False, mant, exp, n)
    quotient(f"{name}, negative", tags + ["sign"], True, mant, exp, n)


quotient("0 / 0 is NaN", ["zero-divisor", "zero"], False, 0, 0, 0)
quotient("1 / 0 is +∞", ["zero-divisor", "overflow"], False, 1, 0, 0)
quotient("-1 / 0 is -∞", ["zero-divisor", "overflow", "sign"], True, 1, 0, 0)
quotient("0 / 3 is zero", ["zero", "exact"], False, 0, 0, 3)
both_sign_quotients("7 / 1", ["exact"], 7, 0, 1)
both_sign_quotients("1 / 3", ["nearest"], 1, 0, 3)
both_sign_quotients("2 / 3", ["nearest"], 2, 0, 3)
both_sign_quotients("5 / 7", ["nearest"], 5, 0, 7)
both_sign_quotients("10 / 3", ["nearest"], 10, 0, 3)
both_sign_quotients("1 / 10", ["nearest"], 1, 0, 10)
both_sign_quotients("1 / 2^40 is exact", ["exact"], 1, 0, 1 << 40)
both_sign_quotients("3 / 4", ["exact"], 3, 0, 4)
both_sign_quotients("2^64 / 3", ["nearest"], 1, 64, 3)
both_sign_quotients("(2^128 - 1) / (2^64 - 1) is 2^64 + 1, a tie down", ["tie"], (1 << 128) - 1, 0, (1 << 64) - 1)
both_sign_quotients("(2^128 - 1) / 2 rounds up to 2^127", ["nearest"], (1 << 128) - 1, 0, 2)
both_sign_quotients("1 / (2^64 - 1), a quotient below one", ["nearest"], 1, 0, (1 << 64) - 1)
both_sign_quotients("3 · (2^53 + 1) / 3 is the 2^53 + 1 tie, down", ["tie"], 3 * ((1 << 53) + 1), 0, 3)
both_sign_quotients("(3 · (2^53 + 1) + 1) / 3 is above the tie, up", ["sticky"], 3 * ((1 << 53) + 1) + 1, 0, 3)
both_sign_quotients("(3 · (2^53 + 3) - 1) / 3 is below the tie, down", ["nearest"], 3 * ((1 << 53) + 3) - 1, 0, 3)
both_sign_quotients("2^-1074 / 2 ties to zero", ["tie", "subnormal", "zero"], 1, -1074, 2)
both_sign_quotients("2^-1074 / 3 rounds to zero", ["nearest", "subnormal", "zero"], 1, -1074, 3)
both_sign_quotients("3 · 2^-1074 / 2 ties up to 2^-1073", ["tie", "subnormal"], 3, -1074, 2)
both_sign_quotients("7 · 2^-1074 / 4 rounds to 2^-1073", ["nearest", "subnormal"], 7, -1074, 4)
both_sign_quotients("2^-1000 / 2^60 is a subnormal, exact", ["exact", "subnormal"], 1, -1000, 1 << 60)
both_sign_quotients("2^-1000 / (2^60 + 1) is a subnormal, rounded", ["nearest", "subnormal"], 1, -1000, (1 << 60) + 1)
both_sign_quotients("2^1024 / 1 overflows", ["overflow"], 1, 1024, 1)
both_sign_quotients("2^1024 / 2 is 2^1023", ["exact"], 1, 1024, 2)
both_sign_quotients("2^1100 / 2^60 is 2^1040, past the ceiling and infinite", ["overflow"], 1, 1100, 1 << 60)
both_sign_quotients("2^1080 / 2^60 is 2^1020, finite", ["exact"], 1, 1080, 1 << 60)
both_sign_quotients("f64::MAX / (2^64 - 1)", ["nearest"], (1 << 53) - 1, 971, (1 << 64) - 1)
both_sign_quotients("a 131-bit mantissa / (2^63 + 1), across limbs", ["nearest"], (1 << 130) + 1, -96, (1 << 63) + 1)
both_sign_quotients("a 131-bit mantissa / 1, rounded", ["nearest"], (1 << 130) + 1, -96, 1)
both_sign_quotients("a 150-bit mantissa / 2^63, across limbs", ["nearest"], (1 << 150) + (1 << 70) + 1, -96, 1 << 63)
both_sign_quotients("(2^53 + 1) · 2^63 / 2^63 is the tie, down", ["tie"], (1 << 53) + 1, 63, 1 << 63)
both_sign_quotients("((2^53 + 1) · 2^63 + 1) / 2^63 is above the tie, up", ["sticky"], ((1 << 53) + 1 << 63) + 1, 0, 1 << 63)
both_sign_quotients("1 / u64::MAX", ["nearest"], 1, 0, (1 << 64) - 1)
both_sign_quotients("2^100 / u64::MAX", ["nearest"], 1, 100, (1 << 64) - 1)

# --- A spread of pseudo-random dyadics and quotients, seeded so the file is reproducible -------


class SplitMix:
    """splitmix64, the stream tests/support.rs uses, so a vector can be re-derived by hand."""

    def __init__(self, seed: int):
        self.s = seed & (2**64 - 1)

    def next(self) -> int:
        self.s = (self.s + 0x9E3779B97F4A7C15) & (2**64 - 1)
        z = self.s
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & (2**64 - 1)
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & (2**64 - 1)
        return z ^ (z >> 31)


rng = SplitMix(0x5EED)
for i in range(48):
    width = 1 + rng.next() % 140
    mant = rng.next() | (rng.next() << 64) | (rng.next() << 128)
    mant = (mant & ((1 << width) - 1)) | (1 << (width - 1))
    exp = -96 + rng.next() % 150
    negative = rng.next() % 2 == 0
    a = Fraction(mant) * Fraction(2) ** exp
    tags = ["random"]
    if f32_bits_cpython(negative, a) != f32_bits_via_f64(negative, a):
        tags.append("f32-differs")
    read(f"random dyadic {i}: {width} bits at 2^{exp}", tags, negative, mant, exp)
for i in range(32):
    width = 1 + rng.next() % 140
    mant = rng.next() | (rng.next() << 64) | (rng.next() << 128)
    mant = (mant & ((1 << width) - 1)) | (1 << (width - 1))
    exp = -96 + rng.next() % 150
    # A divisor of a random width, inside u64.
    n = max(1, rng.next() >> (rng.next() % 64))
    negative = rng.next() % 2 == 0
    quotient(f"random quotient {i}: {width} bits at 2^{exp} over {n}", ["random"], negative, mant, exp, n)

tag_counts = {}
for v in reads:
    for t in v["tags"]:
        tag_counts[t] = tag_counts.get(t, 0) + 1
quotient_tag_counts = {}
for v in quotients:
    for t in v["tags"]:
        quotient_tag_counts[t] = quotient_tag_counts.get(t, 0) + 1

doc = {
    "generator": "scripts/gen_rounding_vectors.py",
    "python": sys.version.split()[0],
    "reads": reads,
    "quotients": quotients,
}
OUT.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
print(f"{OUT.relative_to(OUT.parents[2])}: {len(reads)} reads {tag_counts}")
print(f"{len(quotients)} quotients {quotient_tag_counts}")
