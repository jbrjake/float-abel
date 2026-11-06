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
