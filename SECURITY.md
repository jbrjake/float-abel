# Security

Report a vulnerability through GitHub's private vulnerability reporting on this repository:
<https://github.com/jbrjake/float-abel/security/advisories/new>. That is the only channel; the crate
publishes no email address, and a public issue is not the place for a report that could be exploited
before it is fixed.

## What counts

The crate's claims are correctness claims, so a counterexample to any of them is a vulnerability
here:

- a decoder (`from_bytes`, or the `serde` or rkyv path) accepting bytes that are not a canonical
  value, or an engine seeing two words that are `==` but not numerically equal;
- a group operation landing different bits for the same multiset under a different order, merge
  tree, worker count or architecture, or a retraction leaving residue;
- a read (`to_f64`, `to_f32`, `div_f64`) that is not the correctly rounded value of the exact state;
- a panic on input other than the named `Exact` cap, or a wrapped sum where a cap or a budget should
  have fired;
- `unsafe` outside `src/simd/`, or a SIMD kernel whose bits differ from the scalar reference;
- an allocation, or a size taken from untrusted input without a ceiling.

## What to send

The input that shows it: a multiset, a byte string, or a fuzz artefact, with the tier, the feature
set, the toolchain and the target. Every crash the fuzz targets find is committed as a regression
test, and a report is handled the same way.

## Supported

The crate is unpublished and `main` is the only line. From the first release on, the latest
published minor is supported, and a fix ships as a patch release with a `RUSTSEC` advisory where one
applies.
