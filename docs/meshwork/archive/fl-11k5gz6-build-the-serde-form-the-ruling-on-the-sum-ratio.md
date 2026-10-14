---
id: fl-11k5gz6
title: "Build the serde form the ruling on the SUM ratio picks — serialize_bytes with the format's length prefix or the 7-element field tuple — red in tests/wire.rs first, then serde_impls.rs, DESIGN.md § Wire format and FORMAT.md § serde, and re-take the wire and end-to-end rows on both machines"
status: done
category: crate/differential
labels: [reproducible-floats]
discovered-from: fl-s9hcne6
relates: [fl-s9hcne6, fl-bbevtyv]
verify: "all(lacks BENCH.md /commit a22fae7/, run cargo test package=float-abel wire_)"
docs:
  - docs/DESIGN.md#wire-format-sp-wire
  - FORMAT.md#serde
  - BENCH.md#the-wire-form
seq: 145
created: 2026-10-14T13:21Z
covers:
  - ref: docs/DESIGN.md#sp-wire
    sha: a91277dc1dac9b46521d0d088b4442aa4c3bc85a6d5ec5ad8921385c74f6a30d
---

Contingent on fl-bbevtyv: build whichever serde form the owner picks there, or drop this task if
the ruling is (a) accept or (c) revisit the layout. The numbers that decide it are BENCH.md § The
wire form (benches/wire.rs, commit a22fae7): per update, timely's three bincode passes cost the
33-element `u8` tuple 47.8 ns on NEON and 115.6 on AVX2 against `isize`'s 8.7 and 16.2; one
`serialize_bytes` call would cost 14.8 and 33.5, a 7-element tuple of the fields 26.8 and 55.8.

(b1) `serialize_bytes`: `Serialize for Exact` hands `to_bytes()` to `serialize_bytes`, and the
visitor takes `visit_bytes` and `visit_borrowed_bytes` through `from_bytes`, keeping `visit_seq` so
serde_json's array of numbers still decodes. Every format then frames the 33 bytes with its own
length: bincode 1 an 8-byte `u64` (41 bytes), postcard a 1-byte varint (34). `Wide` and `Fixed`
follow for one rule across the types, or stay, with the module doc saying why.

(b2) the field tuple: `serialize_tuple(7)` over the version byte, `lo`, `hi`, `exp`, `nan`,
`pos_inf`, `neg_inf`, and a visitor that reads the seven and calls `Exact::canonical` directly.
bincode's fixed-int bytes stay `to_bytes` exactly (the bench asserts it); postcard's become
varints, so the postcard-equals-`to_bytes` assertion in tests/wire.rs goes.

Either way: red first in tests/wire.rs (the postcard and serde_json assertions on the new framing,
the short-buffer and non-canonical refusals through the new visitor), then src/wire/serde_impls.rs
and its module doc, DESIGN.md § Wire format's clause (the serde paragraph, re-pinned with `cover`
if the clause is covered), FORMAT.md's serde note if it has one, and the bench's newtype of the
chosen form retired in favour of the crate's impl. Then re-take BENCH.md § The wire form and § The
end-to-end SUM on both machines at the new commit, and move "What reproducibility costs" with it.

## log
- 2026-10-14T13:21Z created
- 2026-10-14T13:30Z open→doing — claimed by claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)
- 2026-10-14T13:36Z cover docs/DESIGN.md#sp-wire @a91277dc1dac
- 2026-10-14T16:42Z doing→done — verify exit 0 @ 6be0b49+5

## comments
- 2026-10-14T13:30Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Owner, 2026-10-14, in session: "if there is a way we can avoid it by making a small modification to the design that doesn't impact capabilities and does improve performance, i am all for it" (the full quote is on fl-bbevtyv). Building (b1), serialize_bytes, on every type: the larger saving, and one rule across the types.
- 2026-10-14T16:42Z [claude (16e02ca3-15ad-4c80-bfb4-53e89fee757d)] Built as (b1), the byte string, at 4153c01: Serialize for Exact and Wide hand to_bytes() to
  serialize_bytes; the visitors take visit_bytes (through from_bytes) and visit_seq (serde_json's
  array of numbers); Fixed keeps the tuple form because its length 1 + 8·LIMBS cannot be spelled as
  a const in the impl and LIMBS has no upper bound. Red first in tests/wire.rs: the postcard framing
  test (a varint length then to_bytes, 34 bytes Exact and 299 Wide; Fixed is to_bytes itself) and
  the refusals (non-canonical, short and long bodies as SerdeDeCustom, a truncated buffer as
  DeserializeUnexpectedEnd). DESIGN.md § Wire format's serde paragraph rewritten and re-pinned
  (cover @a91277dc1dac), FORMAT.md § serde rewritten (41 bytes Exact and 305 Wide on bincode 1, 34
  and 299 on postcard). Measured in BENCH.md § The wire form and § The end-to-end SUM at 2a82388
  (commit 6be0b49): Exact 14.7 ns an update on NEON and 33.3 on AVX2 against the 33-element tuple's
  85.2 and 157.4 and isize's 8.1 and 16.3. The bench's AsTuple newtype stays as the row for the
  tuple form; iterating the bytes as a slice as the crate's impl did (2a82388), it still runs about
  1 ns a call slower per element than that impl did in place, a codegen difference not chased.
