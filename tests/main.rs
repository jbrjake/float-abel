//! The one integration-test target. Each topic is a flat file under `tests/` and a `mod` here, so
//! the suite links once. `autotests = false` in the manifest keeps a topic file from becoming its
//! own binary, and scripts/smoke.sh checks that every topic file is named below.

mod posture;
