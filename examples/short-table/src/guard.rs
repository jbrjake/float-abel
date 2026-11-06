//! One guard around every call into a column: a panic becomes [`Shortfall::Panicked`] with its
//! message and a location trimmed of the machine's paths, under a hook that prints nothing.

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

use crate::column::{Outcome, Shortfall};

thread_local! {
    /// Set while a guarded closure runs, so the hook knows whose panic it is seeing.
    static GUARDED: Cell<bool> = const { Cell::new(false) };
    /// Where the last guarded panic happened.
    static LOCATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

static HOOK: Once = Once::new();

/// Installs the hook once. Inside a guard it records the location and prints nothing; outside
/// one it is the hook that was there before, so a failing test still prints its message.
pub(crate) fn install() {
    HOOK.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if GUARDED.with(Cell::get) {
                let at = info.location().map(|l| trim(l.file(), l.line()));
                LOCATION.with(|c| *c.borrow_mut() = at);
            } else {
                previous(info);
            }
        }));
    });
}

/// `<crate>-<version>/<path>:<line>` for a crate from the registry; the path as rustc recorded
/// it for anything else.
fn trim(file: &str, line: u32) -> String {
    const REGISTRY: &str = "/registry/src/";
    let path = match file.find(REGISTRY) {
        Some(i) => {
            let rest = &file[i + REGISTRY.len()..];
            rest.find('/').map_or(rest, |j| &rest[j + 1..])
        }
        None => file,
    };
    format!("{path}:{line}")
}

/// Runs `f`, turning a panic inside it into [`Shortfall::Panicked`].
pub(crate) fn probe<T>(f: impl FnOnce() -> Outcome<T>) -> Outcome<T> {
    install();
    GUARDED.with(|g| g.set(true));
    LOCATION.with(|c| *c.borrow_mut() = None);
    let caught = panic::catch_unwind(AssertUnwindSafe(f));
    GUARDED.with(|g| g.set(false));
    match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "a payload that is not a string".to_owned());
            let at = LOCATION
                .with(|c| c.borrow_mut().take())
                .unwrap_or_else(|| "an unknown location".to_owned());
            Err(Shortfall::Panicked { message, at })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_registry_path_is_trimmed_to_the_crate_and_its_file() {
        assert_eq!(
            trim(
                "/Users/someone/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accurate-0.4.1/src/sum/ifastsum.rs",
                89
            ),
            "accurate-0.4.1/src/sum/ifastsum.rs:89"
        );
        assert_eq!(trim("src/main.rs", 3), "src/main.rs:3");
    }

    #[test]
    fn a_panic_becomes_panicked_with_its_message_and_a_trimmed_location() {
        let line = line!() + 1;
        let out: Outcome<()> = probe(|| panic!("the message"));
        match out {
            Err(Shortfall::Panicked { message, at }) => {
                assert_eq!(message, "the message");
                assert_eq!(at, format!("src/guard.rs:{line}"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_formatted_panic_message_is_kept_whole() {
        let out: Outcome<()> = probe(|| panic!("value {}", 1 + 1));
        assert!(matches!(out, Err(Shortfall::Panicked { message, .. }) if message == "value 2"));
    }

    #[test]
    fn a_closure_that_returns_passes_its_outcome_through() {
        assert_eq!(probe(|| Ok(7)), Ok(7));
        assert_eq!(
            probe(|| Err::<(), _>(Shortfall::Absent("x".into()))),
            Err(Shortfall::Absent("x".into()))
        );
    }
}
