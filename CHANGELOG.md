# Changelog

One section per release, newest first, written by `scripts/changelog-entry.sh` from the
conventional-commit log since the previous release tag and grouped by commit type; a breaking change
is marked. The version is counted from 0.0.0 over every commit since the first (DESIGN.md § Release),
and every release on GitHub carries the `.crate` with its provenance attestation (README § getting it).

## [Unreleased]

## [0.30.2](https://github.com/jbrjake/float-abel/compare/v0.0.0...v0.30.2) - 2026-11-02

### Added
- *(release)* count the version from 0.0.0 over every commit since the first and cut releases from it with the repository's own scripts
- *(release)* cut releases with release-plz from the log and attach the attested .crate to every GitHub Release
- *(docs)* rewrite README
- *(ci)* publish to crates.io from release.yml on the owner's workflow_dispatch
- *(dbsp)* give Wide and Fixed DBSP's weight traits
- *(differential)* give Wide and Fixed differential-dataflow's weight traits
- *(laws)* add the closure law, check_closure, with a compact-only Exact as its control
- *(differential)* ship the banded SUM idiom as Exact::band and BandedSum::banded_sum
- *(exact)* encode the heap form on the wire and in the rkyv archive as its block behind a tag
- *(exact)* restore the heap arm as the 32-byte niche form behind an alloc feature
- *(wire)* serialise Exact and Wide as one byte string under serde
- *(arbitrary)* implement Arbitrary for Exact, Wide and Fixed
- *(dbsp)* adapt Exact to dbsp 0.364's weight traits behind the dbsp feature, with the UDA example
- *(simd)* add the AVX2 kernel, four doubles a step behind runtime detection
- *(simd)* fold every tier's bulk entries in lane-parallel tiles, with a portable kernel on every target and a NEON kernel behind the simd feature
- *(binned)* deposit the 106-bit product onto the bins, truncated below the unit and refused at Wide's ceiling, with a bulk entry that checks every pair first
- *(golden)* hash every tier's bytes, reads and refusals on the vectors and 10⁴ seeded multisets, commit the claim, and assert it on seven targets
- *(binned)* build Binned<K> as independent carry-save bins on Wide's limb grid, with Add, into_wide, Wide::to_binned, and compile-fail cases that keep it out of the engine weight traits
- *(differential)* adapt Exact to differential-dataflow 0.25 as a diff type, prove it on three workers with serialized exchange, and ignore the closure's two unmaintained advisories by id
- *(laws)* publish the laws kit behind the laws feature and run every exact tier and a naive f64 control through it
- *(wire)* add the canonical wire format with to_bytes and from_bytes on every tier, an Exact decoder that refuses non-canonical words by name, serde carrying the same bytes, and FORMAT.md with vectors
- *(reads)* add to_f32 on every tier and prove to_f64, to_f32 and div_f64 against CPython-generated vectors
- *(types)* convert between the tiers with Wide from Fixed exact under a const assertion, Exact from Wide and Fixed refused past the width, and Wide from Exact refused outside the window
- *(types)* certify Wide and Fixed as repr(C) plain words with pinned layouts and safe constructors from their limbs
- *(exact)* make Exact a 32-byte Copy plain word with i32 special counts and the integer-amount ring, and drop the heap arm and the alloc feature
- *(exact)* publish Exact on every feature set, with the named cap on the operators and a checked API that returns it
- *(exact)* bound the exponent at 2^30 in every build, and tell an exponent overflow apart from a promotion
- *(exact)* add the compact tier, a canonical 127-bit dyadic with checked add and mul, total negation, numeric Ord and the reads, behind the alloc feature
- *(wide)* add Wide, the 34-limb exact f64 sum with signed special counters, total From<f64>, group ops, Mul<i64>, read-time projection, deposit_product and bitrep limb conformance
- *(fixed)* Fixed<LSB, LIMBS> — exact-or-refused deposit, quantize, the group ops, Ord and Hash, to_f64 and div_f64

### Fixed
- *(release)* anchor the include globs to the crate root
- *(release)* let the lockfile update read the crates.io index, which a runner has not cached
- *(release)* install release-plz 0.4.3 by its tarball's sha256, since 0.4.2's musl build fails on the changelog header
- *(release)* keep release-plz's JSON output out of the working tree it refuses to find dirty
- *(release)* fail the release-plz jobs on release-plz's own exit status and let the release job read pull requests
- *(ci)* let a commit scope carry a comma, as refactor(wide,fixed) does
- *(deps)* clear RUSTSEC-2026-0385, -0391, -0398 and -0403 from the dbsp closure
- *(deps)* move dbsp's floor to 0.367 past zip 6's three advisories
- *(ci)* skip the task store's lint under CI, where no job installs meshwork
- *(exact)* close the limb cap under retraction at its edge
- *(exact)* let the compact add decide on the sum's span, not on the raised operand or the aligned frame
- *(noalloc)* fold the zero with the assign operator so the workspace clippy leg passes
- *(types)* assert the headroom and counter budgets in every build on Wide and Fixed, and run the panic tests under --release in the push gate

### Performance
- *(exact)* inline the operator chain down to the compact tier

### Changed
- *(wide,fixed)* move the operator impls into ops submodules
- *(simd)* delete the floating-point Binned kernel, the loser of the A/B, and record the numbers

### Documentation
- *(release)* record the release ruling: release-plz on dev→main from a v0.0.0 base, an attested .crate in place of a signed tag, crates.io outside the release
- *(changelog)* name the first release by the log, not a number
- *(claude)* record the advisory-trigger and macOS-leg rulings
- *(bench)* re-take the crossover at f24b051 on both machines
- *(bench)* re-take the reproducibility-cost table at f24b051 on both machines
- *(bench)* record the heap arm's budget A/B against the Copy word
- *(bench)* record the Wide row of the end-to-end SUM
- *(overflow)* replace the options and their recommendation with what was decided
- *(exact)* state the heap form's contract in the README and the design
- *(design)* state the heap form's diff row as accepted at 1.9–2.4× beside the Copy word's
- *(overflow)* measure the 32-byte niche form against the Copy word, the enum and the banded idiom on both exchanges, and set the heap arm's budget
- *(overflow)* measure the heap arm as a proxy beside the banded idiom on three bands, and list it as an option with what it reopens
- *(overflow)* record the spike's findings — the two failure classes, the boundary bug, the banded idiom and its measured cost
- *(design)* state the diff row as accepted and the product kernel as held for a consumer
- *(design)* return the diff row and the product sentence to open rulings
- *(readme)* write the outward docs and run the README's Rust blocks as doctests
- *(design)* restate the targets and the diff row to the figures the owner accepted
- *(bench)* record the crossover on both machines and write the decision memo
- *(bench)* re-take every table at 2a82388 and name each section's commit
- *(bench)* record the wire form's three bincode passes per update on both machines
- *(bench)* record the prototype table, the width caps, the product row and the end-to-end SUM
- *(design)* state that deposit_product is refused by name on Fixed and Wide outside the window, and that Exact holds every product
- *(design)* finish the heap-era wording — the word's size, the width cap in the bounds and the kill table, and the dev-dependency list
- *(dbsp)* pin dbsp 0.364.0 on rustc 1.99.0

### Tests
- *(exact)* cross six of the eleven cap-edge shapes under miri so the miri job keeps its headroom
- *(binned)* poison the slices' last entry by their length so the refusal case runs under miri
- *(rounding)* scale the random-multiset slice through rounds() so miri runs 200
- *(laws)* run the ring law at the limb cap's edge
- *(laws)* draw the limb cap's edge in the kit's Exact strategy
- *(laws)* run the four older laws on the full-range Exact strategy
- *(engines)* prove the five histories and product folds on the heap Exact inside differential and DBSP
- *(ui)* pin the compile-fail wording to stable 1.99 with rust-src and skip it elsewhere
- *(miri)* run the suite under miri with bit-built powers of two and scaled sweeps
- *(fuzz)* add the decode, sum-vs-oracle and sub-round-trip cargo-fuzz targets on a num-bigint oracle
- *(golden)* run the transcript with simd on and record every bulk entry, so the hash covers each kernel
- *(wire)* draw random limbs with array::from_fn so the topic stays under the length target
- *(wire)* prove the serde bytes with postcard instead of unmaintained bincode, drop the advisory ignore, and allow Unicode-3.0 for unicode-ident

### Benchmarks
- *(baseline)* commit the per-machine bench baseline and its script
- *(dd_sum)* add the Wide row on the same-scale fixture
- *(dd_sum)* record the banded idiom beside the plain one on the same-scale and mixed fixtures
- *(dd_sum)* re-take the end-to-end SUM on the heap form beside isize and read the row against the kill line
- *(crossover)* price one change as the group grows, Exact in differential against a plain f64 re-sum
- *(wire)* iterate the tuple form's bytes as a slice, as the crate's impl of it did
- *(wire)* time timely's three bincode passes on a batch of Exact diffs against isize and two candidate forms
- *(dd_sum)* run the end-to-end SUM on typed exchange beside serialized
- add the ops and dd_sum targets and the products row
- *(simd)* build Binned kernel (ii), the floating-point peel, beside kernel (i) and hold it to the fold's bits
- *(simd)* measure each tier's bulk entry against its scalar reference on NEON and AVX2, and start BENCH.md

### Build
- *(deps)* update rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.10 past RUSTSEC-2026-0382 and -0383
- *(deps)* bump rustls to 0.23.46 past four advisories in dbsp's closure
- *(dbsp)* take dbsp on a caret like differential's
- *(deny)* check the dbsp closure with its own deny-dbsp.toml and keep deny.toml to the crate's other features

### CI
- *(release)* cut the release as one pull request, dev into main
- *(gates)* run the release scripts' self-tests in the commit and push gates
- *(release)* cut the release PR from a clone with scripts/release-pr.sh, not a runner
- *(actions)* bump actions/upload-artifact from 4.6.2 to 7.0.2
- *(actions)* bump actions/cache from 4.3.0 to 6.1.0
- *(actions)* bump rust-lang/crates-io-auth-action from 1.0.5 to 1.0.6
- *(actions)* bump actions/checkout from 4.4.0 to 7.0.1
- *(workflows)* fold five workflows into three, the suite nightly on dev only when it moved and again on the pull request into main
- *(release)* run no ci on a push to dev and open the release PR on the owner's dispatch alone
- *(release)* refuse the release PR while the default branch is not dev
- *(release)* gate commit subjects on the Conventional Commits form, in the commit-msg hook, both gates and the PR
- *(release)* run ci on dev and point the push and release-cut commands at it
- *(deny)* fail both gates when an ignored advisory no longer matches a crate
- *(dbsp)* free the runner's disk and build a changed lockfile from a clean target
- *(actions)* let dependabot refresh the action hashes by PR
- *(golden)* check the wasmtime and cross tarballs against a sha256 before extracting
- *(security)* pin every action to its commit hash and fail the gates on a tag

### Other
- *(deny)* ignore RUSTSEC-2025-0141 for bincode 1.3.3 by id, the adapter's wire and a dev-dependency its team calls complete
