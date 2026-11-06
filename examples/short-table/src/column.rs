//! What every column implements, what a probe hands back, and the recipes states are built from.

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

/// One crate's accumulator, driven the way an incremental engine drives a weight. Each row adds at
/// most one method here and implements it in every column, so a row lands as one diff that reads
/// "here is what this property means for each of these crates". No method has a default: a crate
/// that lacks one says so, as [`Shortfall::Absent`], beside a `const` assertion that the trait
/// really is missing.
pub(crate) trait Column {
    /// The column's heading in the table.
    const NAME: &'static str;
    /// The crate's accumulator.
    type State;
    /// The crate's empty accumulator.
    fn empty() -> Self::State;
    /// Add `x`, through the crate's entry for one value.
    fn insert(s: &mut Self::State, x: f64) -> Outcome<()>;
    /// Remove `x`, through the crate's own inverse where it has one.
    fn retract(s: &mut Self::State, x: f64) -> Outcome<()>;
    /// The value, as the crate reads it out. `&mut` because xsum's `sum` wants it.
    fn read(s: &mut Self::State) -> Outcome<f64>;
    /// The state as the crate itself shows it: its bytes, or `Debug` where that is the only
    /// window.
    fn view(s: &Self::State) -> Outcome<View>;
    /// The crate's entry for a whole slice, when it has one; `None` is not a shortfall.
    fn batch(xs: &[f64]) -> Option<Outcome<f64>>;
    /// The one state that holds everything two workers' states hold.
    fn merge(a: Self::State, b: Self::State) -> Outcome<Self::State>;
    /// The state's negative, from the state alone.
    fn negate(s: Self::State) -> Outcome<Self::State>;
    /// The crate's `==` on two states.
    fn eq(a: &Self::State, b: &Self::State) -> Outcome<bool>;
    /// Whether two states hash alike under the crate's `Hash`.
    fn hash_eq(a: &Self::State, b: &Self::State) -> Outcome<bool>;
    /// The crate's `is_zero`, or `== empty` where it has `==` and no `is_zero`.
    fn is_zero(s: &Self::State) -> Outcome<bool>;
}

/// Whether two values hash alike under the standard hasher. Compared here and never printed,
/// because the hasher's output is not stable across Rust releases.
pub(crate) fn hashes_equal<T: Hash + ?Sized>(a: &T, b: &T) -> bool {
    fn one<T: Hash + ?Sized>(v: &T) -> u64 {
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        h.finish()
    }
    one(a) == one(b)
}

/// Whether `x` is a zero of either sign, decided on the bits.
pub(crate) fn is_zero(x: f64) -> bool {
    x.to_bits() << 1 == 0
}

/// What a probe hands back: the passing reading's description, or why it did not pass.
pub(crate) type Outcome<T> = Result<T, Shortfall>;

/// Why a probe did not pass. Each variant prints as one of the four evidence kinds; a probe that
/// ran and read a wrong value is [`Shortfall::Wrong`], printed as `ran:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Shortfall {
    /// The probe ran and the value or comparison was wrong.
    Wrong(String),
    /// The crate's API declined: an `Option::None`, a `false`.
    Refused(String),
    /// The crate panicked; the location is trimmed to `<crate>-<version>/<path>:<line>`.
    Panicked {
        /// The panic's message.
        message: String,
        /// Where it panicked.
        at: String,
    },
    /// The API does not exist.
    Absent(String),
}

impl fmt::Display for Shortfall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shortfall::Wrong(s) => write!(f, "ran: {s}"),
            Shortfall::Refused(s) => write!(f, "refused: {s}"),
            Shortfall::Panicked { message, at } => write!(f, "panicked: `{message}` at {at}"),
            Shortfall::Absent(s) => write!(f, "absent: {s}"),
        }
    }
}

/// A state as its crate shows it, so two states can be compared the way the crate's own `==`
/// or serialisation would compare them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum View {
    /// The crate's own byte encoding.
    Bytes(Vec<u8>),
    /// The crate's `Debug` output, or a rendering of its parts, where it has no bytes.
    Debug(String),
}

impl View {
    /// How two views of the same kind differ, for an evidence line: both renderings when they
    /// are short, else where they part and a window of each around that point.
    pub(crate) fn difference(&self, other: &View) -> String {
        match (self, other) {
            (View::Bytes(a), View::Bytes(b)) => match a.iter().zip(b).position(|(x, y)| x != y) {
                Some(i) => format!("bytes differ from offset {i} of {}", a.len()),
                None => format!("bytes differ in length, {} against {}", a.len(), b.len()),
            },
            (View::Debug(a), View::Debug(b)) => {
                if a.chars().count() + b.chars().count() <= 64 {
                    format!("`{a}` against `{b}`")
                } else {
                    let (x, y): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
                    let at = x
                        .iter()
                        .zip(&y)
                        .position(|(p, q)| p != q)
                        .unwrap_or(x.len().min(y.len()));
                    format!(
                        "Debug differs from character {at}: {} against {}",
                        window(&x, at),
                        window(&y, at)
                    )
                }
            }
            _ => "the two views are of different kinds".to_owned(),
        }
    }
}

/// Up to 24 characters of `chars` around `at`, in backticks with an ellipsis on each cut side.
fn window(chars: &[char], at: usize) -> String {
    let start = at.saturating_sub(8);
    let end = (start + 24).min(chars.len());
    let body: String = chars[start..end].iter().collect();
    format!(
        "`{}{body}{}`",
        if start > 0 { "…" } else { "" },
        if end < chars.len() { "…" } else { "" }
    )
}

/// One step of a recipe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Step {
    /// [`Column::insert`] of the value.
    Insert(f64),
    /// [`Column::retract`] of the value.
    Retract(f64),
}

/// The state a recipe builds in column `C`, stopping at the first step the crate refuses.
pub(crate) fn build<C: Column>(recipe: &[Step]) -> Outcome<C::State> {
    let mut s = C::empty();
    for step in recipe {
        match *step {
            Step::Insert(x) => C::insert(&mut s, x)?,
            Step::Retract(x) => C::retract(&mut s, x)?,
        }
    }
    Ok(s)
}

/// The value as the log prints it: Rust's shortest round-trip form, in exponent form from a
/// billion up (where `Debug` would still spell out `1e12`), with the specials and the two named
/// edges of the range spelled by name.
pub(crate) fn num(x: f64) -> String {
    // Bit identity, not `==`: the one comparison that tells the named values from their
    // neighbours and never confuses `-0.0` with `0.0`.
    let is = |y: f64| x.to_bits() == y.to_bits();
    if x.is_nan() {
        "NaN".to_owned()
    } else if is(f64::INFINITY) {
        "+∞".to_owned()
    } else if is(f64::NEG_INFINITY) {
        "−∞".to_owned()
    } else if is(f64::MAX) {
        "f64::MAX".to_owned()
    } else if is(f64::MIN_POSITIVE) {
        "f64::MIN_POSITIVE".to_owned()
    } else if x.abs() >= 1e9 {
        format!("{x:e}")
    } else {
        format!("{x:?}")
    }
}

/// A recipe as the log prints it: `+x` for an insert, `−x` for a retract, a negative value in
/// parentheses, and a run of one step collapsed to `+x ×n`.
pub(crate) fn recipe(steps: &[Step]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < steps.len() {
        let step = steps[i];
        let mut n = 1;
        while i + n < steps.len() && same_step(steps[i + n], step) {
            n += 1;
        }
        let (sign, x) = match step {
            Step::Insert(x) => ('+', x),
            Step::Retract(x) => ('−', x),
        };
        let value = if x.is_sign_negative() {
            format!("({})", num(x))
        } else {
            num(x)
        };
        out.push(if n == 1 {
            format!("{sign}{value}")
        } else {
            format!("{sign}{value} ×{n}")
        });
        i += n;
    }
    out.join(" ")
}

/// Bit-for-bit equality of two steps, so a run of NaNs is a run too.
fn same_step(a: Step, b: Step) -> bool {
    match (a, b) {
        (Step::Insert(x), Step::Insert(y)) | (Step::Retract(x), Step::Retract(y)) => {
            x.to_bits() == y.to_bits()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_in_their_shortest_form_and_the_edges_by_name() {
        assert_eq!(num(0.1), "0.1");
        assert_eq!(num(1e-20), "1e-20");
        assert_eq!(num(1e30), "1e30");
        assert_eq!(num(1e12), "1e12");
        assert_eq!(num(-1e16), "-1e16");
        assert_eq!(num(1e308), "1e308");
        assert_eq!(num(65536.0), "65536.0");
        assert_eq!(num(5e-324), "5e-324");
        assert_eq!(num(f64::MAX), "f64::MAX");
        assert_eq!(num(f64::MIN_POSITIVE), "f64::MIN_POSITIVE");
        assert_eq!(num(f64::NAN), "NaN");
        assert_eq!(num(f64::INFINITY), "+∞");
        assert_eq!(num(f64::NEG_INFINITY), "−∞");
    }

    #[test]
    fn recipes_print_each_step_and_collapse_runs() {
        use Step::{Insert, Retract};
        assert_eq!(recipe(&[Insert(0.5), Insert(0.5)]), "+0.5 ×2");
        assert_eq!(recipe(&[Insert(0.1), Retract(0.1)]), "+0.1 −0.1");
        assert_eq!(recipe(&[Insert(1.0), Insert(-1e16)]), "+1.0 +(-1e16)");
        assert_eq!(recipe(&[Retract(f64::NAN)]), "−NaN");
        assert_eq!(recipe(&vec![Insert(1.0); 65536]), "+1.0 ×65536");
        assert_eq!(recipe(&[]), "");
    }

    #[test]
    fn shortfalls_print_as_the_four_kinds() {
        assert_eq!(
            Shortfall::Wrong("reads 0.0".into()).to_string(),
            "ran: reads 0.0"
        );
        assert_eq!(
            Shortfall::Refused("is None".into()).to_string(),
            "refused: is None"
        );
        assert_eq!(
            Shortfall::Panicked {
                message: "boom".into(),
                at: "x-1.0.0/src/lib.rs:1".into()
            }
            .to_string(),
            "panicked: `boom` at x-1.0.0/src/lib.rs:1"
        );
        assert_eq!(
            Shortfall::Absent("no Neg".into()).to_string(),
            "absent: no Neg"
        );
    }

    #[test]
    fn view_differences_say_where_or_show_both() {
        let a = View::Bytes(vec![1, 2, 3]);
        assert_eq!(
            a.difference(&View::Bytes(vec![1, 9, 3])),
            "bytes differ from offset 1 of 3"
        );
        assert_eq!(
            a.difference(&View::Bytes(vec![1, 2, 3, 4])),
            "bytes differ in length, 3 against 4"
        );
        let short = View::Debug("[0.0]".into());
        assert_eq!(
            short.difference(&View::Debug("[]".into())),
            "`[0.0]` against `[]`"
        );
        let long = View::Debug(format!("{}A{}", "x".repeat(20), "x".repeat(20)));
        assert_eq!(
            long.difference(&View::Debug("x".repeat(41))),
            "Debug differs from character 20: `…xxxxxxxxAxxxxxxxxxxxxxxx…` against \
             `…xxxxxxxxxxxxxxxxxxxxxxxx…`"
        );
        let prefix = View::Debug("y".repeat(40));
        assert_eq!(
            prefix.difference(&View::Debug("y".repeat(50))),
            "Debug differs from character 40: `…yyyyyyyy` against `…yyyyyyyyyyyyyyyyyy`"
        );
    }
}
