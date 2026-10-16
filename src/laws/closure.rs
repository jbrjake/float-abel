//! [`check_closure`]: the operators return on the strategy's values, which is the law a type
//! with a cap breaks.

use core::any::Any;
use core::fmt::Debug;
use core::ops::Mul;
use std::format;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::string::String;
use std::vec::Vec;

use proptest::arbitrary::any;
use proptest::collection::vec;
use proptest::strategy::Strategy;
use proptest::test_runner::{TestError, TestRunner};

use super::report::{fold, Broken, Rng};
use super::{Weight, MULTIPLIERS, SHUFFLES, WEIGHTS};

/// What [`check_closure`] draws per case: the weights, the multipliers each is scaled by, and the
/// seed of the orders the fold runs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureCase<T, K> {
    /// Between one and [`WEIGHTS`] of them, from the weight strategy.
    pub weights: Vec<T>,
    /// Between one and [`MULTIPLIERS`] of them, from the multiplier strategy.
    pub multipliers: Vec<K>,
    /// Drives the shuffles of the fold.
    pub seed: u64,
}

/// Checks that the operators return on the strategy's values: `a + b`, `a − b` and `a · k` for
/// every pair and every multiplier a case draws, and the fold of the case's weights in the drawn
/// order and under 1,000 shuffles of it, because a partial sum on the way to a total that fits
/// need not fit itself. Closure is the first group axiom and the one a type with a cap breaks: an
/// engine adds with `plus_equals`, which cannot return an error, so a sum past a cap is a panic
/// inside the dataflow, in one arrival order and not another. The check catches the panic at the
/// operation and reports it as a value with the operands as the witness, so a capped type is seen
/// whether or not it declares its cap, and the failure names the law rather than the message
/// alone.
///
/// The other checks hold on any strategy inside a type's domain, so a strategy drawn inside the
/// domain by construction proves nothing here: run this one on the values the engine will see.
/// `Exact` returns on every sum and product short of its two bounds, `Wide` inside its headroom
/// and `Fixed` inside its window; the compact form alone, which is `Exact` without the `alloc`
/// feature, is not closed, by design, and this law says so on a strategy past its width.
///
/// # Errors
///
/// proptest's minimal failing [`ClosureCase`], under a reason that names the operation that did
/// not return and the panic it ended in.
pub fn check_closure<T, K>(
    runner: &mut TestRunner,
    weight: impl Strategy<Value = T>,
    multiplier: impl Strategy<Value = K>,
) -> Result<(), TestError<ClosureCase<T, K>>>
where
    T: Weight + Mul<K, Output = T>,
    K: Clone + Debug,
{
    let strategy = (
        vec(weight, 1..=WEIGHTS),
        vec(multiplier, 1..=MULTIPLIERS),
        any::<u64>(),
    )
        .prop_map(|(weights, multipliers, seed)| ClosureCase {
            weights,
            multipliers,
            seed,
        });
    runner.run(&strategy, |case| {
        let ClosureCase {
            weights,
            multipliers,
            seed,
        } = &case;
        let mut broken = Broken::default();
        if let Err(witness) = closed(weights, multipliers, *seed) {
            broken.note("closure", || witness);
        }
        broken.report()
    })
}

/// The first operation over the case that did not return, named with its panic, or `Ok` when
/// every one did.
fn closed<T, K>(weights: &[T], multipliers: &[K], seed: u64) -> Result<(), String>
where
    T: Weight + Mul<K, Output = T>,
    K: Clone + Debug,
{
    for a in weights {
        for b in weights {
            returns(|| format!("{a:?} + {b:?}"), || a.clone() + b.clone())?;
            returns(|| format!("{a:?} − {b:?}"), || a.clone() - b.clone())?;
        }
        for k in multipliers {
            returns(|| format!("{a:?} · {k:?}"), || a.clone() * k.clone())?;
        }
    }
    let mut rng = Rng(seed);
    let mut order = weights.to_vec();
    returns(|| format!("the fold of {order:?}"), || fold(&order))?;
    for _ in 0..SHUFFLES {
        rng.shuffle(&mut order);
        returns(|| format!("the fold of {order:?}"), || fold(&order))?;
    }
    Ok(())
}

/// Runs `op`, and names it with the panic it ended in if it did not return.
fn returns<T>(what: impl FnOnce() -> String, op: impl FnOnce() -> T) -> Result<(), String> {
    match catch_unwind(AssertUnwindSafe(op)) {
        Ok(_) => Ok(()),
        Err(panic) => Err(format!(
            "{} did not return: {}",
            what(),
            message(panic.as_ref())
        )),
    }
}

/// The panic's message when its payload is a string, as `panic!` and `assert!` make it.
fn message(panic: &(dyn Any + Send)) -> &str {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("a panic whose payload is not a string")
}
