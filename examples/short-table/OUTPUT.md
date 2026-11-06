# The short table, measured

Written by `cargo run --manifest-path examples/short-table/Cargo.toml`, from this checkout of
float-abel. The neighbours are the versions this package's Cargo.lock pins: bitrep 0.5.1,
xsum 0.1.6, fsum 0.1.4, accurate 0.4.1, num-rational 0.4.2, bigdecimal 0.4.11 and fixed 1.31.0.

| | float-abel | bitrep | xsum | fsum | accurate | num-rational | bigdecimal | fixed |
|---|---|---|---|---|---|---|---|---|
| holds every f64 exactly | yes | yes | yes | yes | yes | yes | yes | no |
| same bits for the same number, however reached | yes | no | no | no | no | yes | no | yes |
| merges partial sums | yes | yes | no | no | yes | yes | yes | yes |
| standalone negatives | yes | no | no | no | no | yes | yes | yes |
| `==` / `Hash` / `is_zero` mean the number | yes | no | no | no | no | yes | yes | yes |
| exact products (explode, join) | yes | no | no | no | no | yes | yes | no |
| retractable NaN and ∞ | yes | no | no | no | no | no | no | no |
| correct on every input tested | yes | yes | no | no | no | yes | yes | no |
| **usable as the weight** | **yes** | **no** | **no** | **no** | **no** | **no** | **no** | **no** |

Each row is a property an incremental engine needs of its weight, DESIGN.md § What an incremental
engine asks of a weight; a column is usable only if every cell in it is yes, and the bold row is
that conjunction, computed. Every cell was measured by running it, and the evidence for each
follows: `ran` says what was read, `refused` that the crate's API declined, `panicked` where the
crate panicked, and `absent` that the API does not exist, which the compiler asserts.

## Evidence

### holds every f64 exactly

The weight carries the column's value, so a carrier that loses any of it makes the `SUM` wrong.
Seven edge doubles, 0.1, 1e-20, 1e30, 1e300, f64::MAX, f64::MIN_POSITIVE, 5e-324: each is read back
alone, and each is entered with the double just below it retracted, which must leave exactly their
gap, one ulp and itself a double. The second check catches a carrier that rounds on entry and rounds
back the same way on read, which the first alone cannot see. Every comparison is of bits. Zero stays
out, because the group's zero is +0.0 and the sign of zero is not a value; NaN and ±∞ have their own
row.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| bitrep | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| xsum | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| fsum | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| accurate | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| num-rational | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| bigdecimal | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| fixed | no | ran: +1e-20 reads 0.0; ran: +1e-20 −9.999999999999998e-21 reads 0.0, want 1.504632769052528e-36; refused: I64F64::checked_from_num(1e30) is None; refused: I64F64::checked_from_num(1e300) is None; refused: I64F64::checked_from_num(f64::MAX) is None; ran: +f64::MIN_POSITIVE reads 0.0; ran: +f64::MIN_POSITIVE −2.225073858507201e-308 reads 0.0, want 5e-324; ran: +5e-324 reads 0.0; ran: +5e-324 −0.0 reads 0.0, want 5e-324 |

### same bits for the same number, however reached

Consolidation adds equal-key diffs in an unspecified order and matches them structurally, so one
number has to be one bit pattern whatever history produced it. Five routes, each two histories of
the same number, compared through the crate's own view of its state, its bytes or its `Debug`: split
(+1.0 against +0.5 ×2); order (+1.0000000000000002 ×2 +1.0 against +1.0000000000000002 +1.0
+1.0000000000000002); cancel (empty against +0.1 −0.1); return (+1.0 against +1.0 +0.5 −0.5);
excursion (+1e-12 against +1e-12 +1e12 −1e12). The excursion takes float-abel through its heap form
and back. A column passes if every route's two views are equal.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: all 5 routes view equal |
| bitrep | no | ran: split: +1.0 against +0.5 ×2, bytes differ from offset 273 of 281 |
| xsum | no | ran: split: +1.0 against +0.5 ×2, Debug differs from character 293: `…ate: 2046, m_inf: 0, m_n…` against `…ate: 2045, m_inf: 0, m_n…`; ran: cancel: empty against +0.1 −0.1, Debug differs from character 278: `…ate: 2047, m_inf: 0, m_n…` against `…ate: 2045, m_inf: 0, m_n…`; ran: return: +1.0 against +1.0 +0.5 −0.5, Debug differs from character 293: `…ate: 2046, m_inf: 0, m_n…` against `…ate: 2044, m_inf: 0, m_n…`; ran: excursion: +1e-12 against +1e-12 +1e12 −1e12, Debug differs from character 298: `…ate: 2046, m_inf: 0, m_n…` against `…ate: 2044, m_inf: 0, m_n…` |
| fsum | no | ran: cancel: empty against +0.1 −0.1, `FSum { partials: [] }` against `FSum { partials: [0.0] }`; ran: excursion: +1e-12 against +1e-12 +1e12 −1e12, `FSum { partials: [1e-12] }` against `FSum { partials: [1e-12, 0.0] }` |
| accurate | no | ran: split: +1.0 against +0.5 ×2, Debug differs from character 20: `…um { i: 1, a1: [0.0, 0.0…` against `…um { i: 2, a1: [0.0, 0.0…`; ran: order: +1.0000000000000002 ×2 +1.0 against +1.0000000000000002 +1.0 +1.0000000000000002, Debug differs from character 5146: `…0.0, 3.0000000000000004,…` against `…0.0, 3.0, 0.0, 0.0, 0.0,…`; ran: cancel: empty against +0.1 −0.1, Debug differs from character 20: `…um { i: 0, a1: [0.0, 0.0…` against `…um { i: 2, a1: [0.0, 0.0…`; ran: return: +1.0 against +1.0 +0.5 −0.5, Debug differs from character 20: `…um { i: 1, a1: [0.0, 0.0…` against `…um { i: 3, a1: [0.0, 0.0…`; ran: excursion: +1e-12 against +1e-12 +1e12 −1e12, Debug differs from character 20: `…um { i: 1, a1: [0.0, 0.0…` against `…um { i: 3, a1: [0.0, 0.0…` |
| num-rational | yes | ran: all 5 routes view equal |
| bigdecimal | no | ran: split: +1.0 against +0.5 ×2, `digits 1 at scale 0` against `digits 10 at scale 1`; ran: cancel: empty against +0.1 −0.1, `digits 0 at scale 0` against `digits 0 at scale 55`; ran: return: +1.0 against +1.0 +0.5 −0.5, `digits 1 at scale 0` against `digits 10 at scale 1` |
| fixed | yes | ran: all 5 routes view equal |

### merges partial sums

Workers sum their shares and the engine combines the partial states. Three workers hold +0.5 +0.25,
+0.125 +1e16 and +1.0 +(-1e16); merged as (w1 ⊕ w2) ⊕ w3 and as w3 ⊕ (w2 ⊕ w1), both must read
1.875, the exact total and itself a double. The split is chosen so that losing the 0.125 under 1e16
anywhere shows: plain f64 reads 0.0 one-shot and merged, and a merge that passes through a rounded
double reads 0.75 in one order and 1.0 in the other. Whether the merged states have equal bits is
the previous row's question.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |
| bitrep | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |
| xsum | no | absent: no merge: the Xsum trait is new, add_list, add, sum and clear, and XsumSmall has no Add<XsumSmall> or AddAssign<XsumSmall> |
| fsum | no | absent: no Add<FSum> or AddAssign<FSum>: FSum takes f64 alone |
| accurate | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |
| num-rational | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |
| bigdecimal | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |
| fixed | yes | ran: (w1 ⊕ w2) ⊕ w3 reads 1.875; w3 ⊕ (w2 ⊕ w1) reads 1.875 |

### standalone negatives

`negate` runs inside `iterate`, `antijoin` and `reduce_abelian`, and consolidation can meet a −x
diff before its +x, so every state needs a negative that exists on its own. S is +0.1 +0.2.
negate(S) must exist without knowing S's inputs; then the history where the negative comes first:
start from negate(S), insert 0.1, insert 0.2, and read exactly zero. Equality with the empty state
is the `==` row's question; this row asks only that the negative exists and cancels exactly.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: negate(+0.1 +0.2), then +0.1 +0.2, reads 0.0 |
| bitrep | no | refused: try_unmerge into an empty SumF64 returned false, and SumF64 has no Neg |
| xsum | no | absent: no Neg on XsumSmall |
| fsum | no | absent: no Neg on FSum |
| accurate | no | absent: no Neg on OnlineExactSum<f64> |
| num-rational | yes | ran: negate(+0.1 +0.2), then +0.1 +0.2, reads 0.0 |
| bigdecimal | yes | ran: negate(+0.1 +0.2), then +0.1 +0.2, reads 0.0 |
| fixed | yes | ran: negate(+0.1 +0.2), then +0.1 +0.2, reads 0.0 |

### `==` / `Hash` / `is_zero` mean the number

Builders dedup by `==`, `is_zero` deletes an update, and `count_total` makes the weight data, so
records merge by comparison and route to workers by `Hash`. Probes: +1.0 and +0.5 ×2 are `==` and
hash equal; +0.1 −0.1 is zero and `==` the empty state; +0.1 and +0.2 are not `==`, so an `Eq` that
says yes to everything fails. A crate with `==` but no `is_zero` is asked whether the state `==` the
empty one. Hashes are from the standard library's default hasher and are compared inside the run,
never printed, because their values are not stable across Rust releases.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: +1.0 == +0.5 ×2; their hashes are equal; +0.1 −0.1 is zero; +0.1 −0.1 == empty; +0.1 != +0.2 |
| bitrep | no | ran: +1.0 != +0.5 ×2 by ==; absent: no Hash on SumF64 |
| xsum | no | absent: no PartialEq on XsumSmall; absent: no Hash on XsumSmall; absent: no PartialEq or Clone on XsumSmall, and sum takes &mut self |
| fsum | no | absent: no PartialEq on FSum; absent: no Hash on FSum |
| accurate | no | absent: no PartialEq on OnlineExactSum<f64>; absent: no Hash on OnlineExactSum<f64>; panicked: `assertion failed: count < xs.len()` at accurate-0.4.1/src/sum/ifastsum.rs:89 |
| num-rational | yes | ran: +1.0 == +0.5 ×2; their hashes are equal; +0.1 −0.1 is zero; +0.1 −0.1 == empty; +0.1 != +0.2 |
| bigdecimal | yes | ran: +1.0 == +0.5 ×2; their hashes are equal; +0.1 −0.1 is zero; +0.1 −0.1 == empty; +0.1 != +0.2 |
| fixed | yes | ran: +1.0 == +0.5 ×2; their hashes are equal; +0.1 −0.1 is zero; +0.1 −0.1 == empty; +0.1 != +0.2 |

### exact products (explode, join)

`explode` multiplies each value by its ±1 diff, and `join` evaluates (a + b) × c as a × c + b × c,
so a weight multiplies exactly, by counts and by other weights. By count, with S = +0.1 +0.2:
scale(S, 3) must read what +0.1 +0.2 +0.1 +0.2 +0.1 +0.2 reads, and scale(S, −1) then +0.1 +0.2 must
read zero. By weight: the square of +1.0000000000009095, less 1.000000000001819, must read
8.271806125530277e-25. The exact square is 1 + 2⁻³⁹ + 2⁻⁸⁰; a product rounded to 53 bits, or
truncated at 64 fractional bits, drops the 2⁻⁸⁰ and reads 0.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: scale(+0.1 +0.2, 3) reads 0.9, as +0.1 +0.2 +0.1 +0.2 +0.1 +0.2 does; scale(+0.1 +0.2, -1), then +0.1 +0.2, reads 0.0; (+1.0000000000009095)², less 1.000000000001819, reads 8.271806125530277e-25 |
| bitrep | no | absent: no Mul<i64> on SumF64; absent: no Mul<SumF64> on SumF64, and DotF64 is a different type |
| xsum | no | absent: no Mul<i64> on XsumSmall; absent: no Mul<XsumSmall> on XsumSmall |
| fsum | no | absent: no Mul<i64> on FSum; absent: no Mul<FSum> on FSum |
| accurate | no | absent: no Mul<i64> on OnlineExactSum<f64>; absent: no Mul<OnlineExactSum<f64>> on OnlineExactSum<f64> |
| num-rational | yes | ran: scale(+0.1 +0.2, 3) reads 0.9, as +0.1 +0.2 +0.1 +0.2 +0.1 +0.2 does; scale(+0.1 +0.2, -1), then +0.1 +0.2, reads 0.0; (+1.0000000000009095)², less 1.000000000001819, reads 8.271806125530277e-25 |
| bigdecimal | yes | ran: scale(+0.1 +0.2, 3) reads 0.9, as +0.1 +0.2 +0.1 +0.2 +0.1 +0.2 does; scale(+0.1 +0.2, -1), then +0.1 +0.2, reads 0.0; (+1.0000000000009095)², less 1.000000000001819, reads 8.271806125530277e-25 |
| fixed | no | ran: (+1.0000000000009095)², less 1.000000000001819, reads 0.0, want 8.271806125530277e-25 |

### retractable NaN and ∞

f64 includes NaN and ±∞, and a `SUM` over a changing column has to recover when the NaN row is
deleted. Four histories, where a retraction is the column's own retraction of that same value: +1.5
+NaN reads NaN, then −NaN reads 1.5; +1.5 +∞ reads ∞, then −∞ reads 1.5; +1.5 +∞ +(−∞) reads NaN,
then −∞ −(−∞) reads 1.5; −NaN +NaN +1.5 reads 1.5. The last starts with a NaN retracted before any
arrived, a state a group holds and IEEE cannot.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: +1.5 +NaN reads NaN, then −NaN reads 1.5; +1.5 +∞ reads ∞, then −∞ reads 1.5; +1.5 +∞ +(−∞) reads NaN, then −∞ −(−∞) reads 1.5; −NaN +NaN +1.5 reads 1.5 |
| bitrep | no | refused: try_unmerge of NaN returned false; refused: try_unmerge of ∞ returned false |
| xsum | no | ran: +1.5 +NaN reads NaN, then −NaN reads NaN, want 1.5; ran: +1.5 +∞ reads ∞, then −∞ reads NaN, want 1.5; ran: +1.5 +∞ +(−∞) reads NaN, then −∞ −(−∞) reads NaN, want 1.5; ran: −NaN +NaN +1.5 reads NaN, want 1.5 |
| fsum | no | ran: +1.5 +NaN reads NaN, then −NaN reads NaN, want 1.5; ran: +1.5 +∞ reads NaN, want ∞; ran: +1.5 +∞ +(−∞) reads NaN, then −∞ −(−∞) reads NaN, want 1.5; ran: −NaN +NaN +1.5 reads NaN, want 1.5 |
| accurate | no | panicked: `assertion failed: count < xs.len()` at accurate-0.4.1/src/sum/ifastsum.rs:89 |
| num-rational | no | refused: BigRational::from_float(NaN) is None; refused: BigRational::from_float(∞) is None |
| bigdecimal | no | refused: BigDecimal::from_f64(NaN) is None; refused: BigDecimal::from_f64(∞) is None |
| fixed | no | refused: I64F64::checked_from_num(NaN) is None; refused: I64F64::checked_from_num(∞) is None |

### correct on every input tested

Every other row assumes the arithmetic underneath is right. Eleven finite sums whose exact answers
are themselves doubles, so no oracle is needed, each run one value at a time through the column's
insert and, where the crate has one, through its entry for a whole slice; both paths must read the
answer. The specials have their own row. The inputs: +0.1 +0.2 +(-0.1) +(-0.2) is 0.0 (the README's
opening example); +1e100 +1.0 +(-1e100) is 1.0 (insert, insert, delete); +1e200 +1e100 +1.0
+(-1e100) +(-1e200) is 1.0 (three scales defeat one correction term); +0.1 ×10 +(-1.0) is
5.551115123125783e-17 (0.1 is 3602879701896397 / 2⁵⁵, so ten of them less one is 2 / 2⁵⁵; a carrier
holding decimal 0.1 reads 0); +1e308 ×2 +(-1e308) is 1e308 (a partial past f64::MAX); +f64::MAX ×2
+(-f64::MAX) is f64::MAX (the same at the top of the range); +1.0 +(-1.0) is 0.0 (a state cancelled
to exactly zero); +5e-324 ×2 is 1e-323 (subnormals); +1e-20 +1.0 +(-1.0) is 1e-20 (below a fixed
grid); +1e30 +2.0 +(-1e30) is 2.0 (above a fixed range); +1.0 ×65536 is 65536.0 (a long stream of
one magnitude).

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: all 11 inputs read their answer, one value at a time and through the slice entry |
| bitrep | yes | ran: all 11 inputs read their answer, one value at a time and through the slice entry |
| xsum | no | ran: the slice entry on +1.0 ×65536 reads -65536.00000000001, want 65536.0 |
| fsum | no | ran: +1e308 ×2 +(-1e308) reads NaN, want 1e308; ran: the slice entry on +1e308 ×2 +(-1e308) reads NaN, want 1e308; ran: +f64::MAX ×2 +(-f64::MAX) reads NaN, want f64::MAX; ran: the slice entry on +f64::MAX ×2 +(-f64::MAX) reads NaN, want f64::MAX |
| accurate | no | panicked: `assertion failed: count < xs.len()` at accurate-0.4.1/src/sum/ifastsum.rs:89 |
| num-rational | yes | ran: all 11 inputs read their answer, one value at a time; no slice entry |
| bigdecimal | yes | ran: all 11 inputs read their answer, one value at a time; no slice entry |
| fixed | no | refused: I64F64::checked_from_num(1e100) is None; refused: I64F64::checked_from_num(1e200) is None; refused: I64F64::checked_from_num(1e308) is None; refused: I64F64::checked_from_num(f64::MAX) is None; ran: +5e-324 ×2 reads 0.0, want 1e-323; ran: +1e-20 +1.0 +(-1.0) reads 0.0, want 1e-20; refused: I64F64::checked_from_num(1e30) is None |

## Control: plain f64

The same probes on plain f64, the type the engines sum with today, so each row's evidence can be
read against what the property costs without an exact weight.

| row | cell | evidence |
|---|---|---|
| holds every f64 exactly | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| same bits for the same number, however reached | no | ran: order: +1.0000000000000002 ×2 +1.0 against +1.0000000000000002 +1.0 +1.0000000000000002, bytes differ from offset 0 of 8; ran: excursion: +1e-12 against +1e-12 +1e12 −1e12, bytes differ from offset 0 of 8 |
| merges partial sums | no | ran: (w1 ⊕ w2) ⊕ w3 reads 0.0, want 1.875; ran: w3 ⊕ (w2 ⊕ w1) reads 0.0, want 1.875 |
| standalone negatives | no | ran: negate(+0.1 +0.2), then +0.1 +0.2, reads -2.7755575615628914e-17, want 0.0 |
| `==` / `Hash` / `is_zero` mean the number | no | absent: no Hash on f64 |
| exact products (explode, join) | no | ran: scale(+0.1 +0.2, -1), then +0.1 +0.2, reads -2.7755575615628914e-17, want 0.0; ran: (+1.0000000000009095)², less 1.000000000001819, reads 0.0, want 8.271806125530277e-25 |
| retractable NaN and ∞ | no | ran: +1.5 +NaN reads NaN, then −NaN reads NaN, want 1.5; ran: +1.5 +∞ reads ∞, then −∞ reads NaN, want 1.5; ran: +1.5 +∞ +(−∞) reads NaN, then −∞ −(−∞) reads NaN, want 1.5; ran: −NaN +NaN +1.5 reads NaN, want 1.5 |
| correct on every input tested | no | ran: +0.1 +0.2 +(-0.1) +(-0.2) reads 2.7755575615628914e-17, want 0.0; ran: the slice entry on +0.1 +0.2 +(-0.1) +(-0.2) reads 2.7755575615628914e-17, want 0.0; ran: +1e100 +1.0 +(-1e100) reads 0.0, want 1.0; ran: the slice entry on +1e100 +1.0 +(-1e100) reads 0.0, want 1.0; ran: +1e200 +1e100 +1.0 +(-1e100) +(-1e200) reads 0.0, want 1.0; ran: the slice entry on +1e200 +1e100 +1.0 +(-1e100) +(-1e200) reads 0.0, want 1.0; ran: +0.1 ×10 +(-1.0) reads -1.1102230246251565e-16, want 5.551115123125783e-17; ran: the slice entry on +0.1 ×10 +(-1.0) reads -1.1102230246251565e-16, want 5.551115123125783e-17; ran: +1e308 ×2 +(-1e308) reads ∞, want 1e308; ran: the slice entry on +1e308 ×2 +(-1e308) reads ∞, want 1e308; ran: +f64::MAX ×2 +(-f64::MAX) reads ∞, want f64::MAX; ran: the slice entry on +f64::MAX ×2 +(-f64::MAX) reads ∞, want f64::MAX; ran: +1e-20 +1.0 +(-1.0) reads 0.0, want 1e-20; ran: the slice entry on +1e-20 +1.0 +(-1.0) reads 0.0, want 1e-20; ran: +1e30 +2.0 +(-1e30) reads 0.0, want 2.0; ran: the slice entry on +1e30 +2.0 +(-1e30) reads 0.0, want 2.0 |
