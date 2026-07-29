//! The thread's ambient tokio hosted event-loop runtime, shared by all
//! `#[wasm_bindgen(tokio)]` exports so their roots run on one runtime: one
//! timer arm, one I/O driver, one keepalive count, and `tokio::spawn` from
//! any of them lands on the same scheduler.

use core::future::Future;
use std::cell::OnceCell;

pub use ::tokio::runtime::HostedRuntime;
pub use ::tokio::task::JoinError;

std::thread_local! {
    static AMBIENT: OnceCell<HostedRuntime> = const { OnceCell::new() };
}

/// Runs `f` with this thread's ambient hosted runtime, building a default
/// one (`enable_all`) on first touch.
pub fn with_ambient<R>(f: impl FnOnce(&HostedRuntime) -> R) -> R {
    AMBIENT.with(|cell| {
        f(cell.get_or_init(|| {
            ::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build_hosted_event_loop_runtime()
                .expect("failed to build ambient tokio hosted runtime")
        }))
    })
}

/// Installs `rt` as this thread's ambient runtime, for callers needing their
/// own builder configuration. Must win the race with the first schedule:
/// errs `rt` back if the ambient is already initialized.
pub fn try_set_ambient(rt: HostedRuntime) -> Result<(), HostedRuntime> {
    AMBIENT.with(|cell| cell.set(rt))
}

/// Schedules `future` as a root on the ambient runtime, delivering its
/// outcome (or a panic, as `Err(JoinError)`) to `on_complete`.
///
/// Called outside any runtime context — a top-level JS call — this drives
/// immediately, so the first poll is synchronous (parity with
/// `future_to_promise`). Called re-entrantly — a task's JS import invoking
/// an export mid-drive — driving on the caller's stack would nest the
/// runtime context, so the root is only queued; the in-progress drive picks
/// it up at its fixed point.
pub fn schedule<F, C>(future: F, on_complete: C)
where
    F: Future + 'static,
    F::Output: 'static,
    C: FnOnce(Result<F::Output, JoinError>) + 'static,
{
    with_ambient(|rt| {
        rt.schedule(future, on_complete);
        if ::tokio::runtime::Handle::try_current().is_err() {
            rt.drive();
        }
    })
}
