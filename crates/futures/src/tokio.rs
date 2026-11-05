//! Tokio event loops for `#[wasm_bindgen(experimental_tokio)]` exports.
//! Only available on the `wasm32-unknown-emscripten` target under
//! `--cfg wasm_bindgen_unstable_tokio` (plus tokio's own `--cfg tokio_unstable`).
//!
//! By default all such exports share the thread's ambient event loop: one
//! timer arm, one I/O driver, one keepalive count, and `tokio::spawn` from
//! any of them lands on the same scheduler. With
//! `experimental_tokio = "isolated"` each invocation instead owns a fresh
//! event loop ([`schedule_isolated`]). A host can also own loops itself
//! ([`schedule_on`]), and a task can hold its loop alive past the root for
//! work that outlives it ([`spawn_extending`]).
//!
//! Combined with `jspi`, the export is a promising activation and the future
//! runs to completion inside it with [`block_on`] / [`block_on_isolated`] on
//! a runtime that parks by JSPI suspension (linked with `-sJSPI`).

#[cfg(not(tokio_unstable))]
compile_error!("`wasm_bindgen_unstable_tokio` requires tokio's `--cfg tokio_unstable`");

extern crate std;

use core::future::Future;
use std::cell::{OnceCell, RefCell};
use std::rc::{Rc, Weak};
use std::vec::Vec;

use ::tokio::runtime::{Handle, Id};
pub use ::tokio::runtime::{LocalEventLoop, Runtime};
pub use ::tokio::task::{JoinError, JoinHandle};

std::thread_local! {
    static AMBIENT: OnceCell<Rc<LocalEventLoop>> = const { OnceCell::new() };
    static AMBIENT_PARKED: OnceCell<Runtime> = const { OnceCell::new() };
    /// Every event loop this module has built or scheduled on, by runtime id, so a
    /// task can find the loop it runs on ([`spawn_extending`]). Dead entries
    /// are pruned on insert.
    static LOOPS: RefCell<Vec<(Id, Weak<LocalEventLoop>)>> = const { RefCell::new(Vec::new()) };
}

/// Builds a hosted event loop with every driver the enabled tokio features
/// provide, as the ambient and isolated loops are, for hosts that own their
/// loops ([`schedule_on`]). The I/O driver needs emscripten's epoll
/// readiness listeners (emscripten-core/emscripten#27547); on an emscripten
/// without them it reports `Unsupported`, and the loop is rebuilt with
/// timers only.
pub fn new_event_loop() -> std::io::Result<LocalEventLoop> {
    match ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build_hosted_local_event_loop(Default::default())
    {
        Err(e) if e.kind() == std::io::ErrorKind::Unsupported => {
            ::tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build_hosted_local_event_loop(Default::default())
        }
        built => built,
    }
}

fn ambient() -> Rc<LocalEventLoop> {
    AMBIENT.with(|cell| {
        cell.get_or_init(|| {
            let rt = Rc::new(new_event_loop().expect("failed to build ambient tokio event loop"));
            register(&rt);
            rt
        })
        .clone()
    })
}

/// Runs `f` with this thread's ambient event loop, building a default one
/// (`enable_all`) on first touch.
pub fn with_ambient<R>(f: impl FnOnce(&LocalEventLoop) -> R) -> R {
    f(&ambient())
}

/// Installs `rt` as this thread's ambient event loop, for callers needing
/// their own builder configuration. Must win the race with the first
/// schedule: errs `rt` back if the ambient is already initialized.
pub fn try_set_ambient(rt: LocalEventLoop) -> Result<(), LocalEventLoop> {
    AMBIENT.with(|cell| {
        if cell.get().is_some() {
            return Err(rt);
        }
        let rt = Rc::new(rt);
        register(&rt);
        let _ = cell.set(rt);
        Ok(())
    })
}

fn register(rt: &Rc<LocalEventLoop>) {
    let id = rt.handle().id();
    LOOPS.with(|loops| {
        let mut loops = loops.borrow_mut();
        if !loops.iter().any(|(i, _)| *i == id) {
            loops.retain(|(_, w)| w.strong_count() > 0);
            loops.push((id, Rc::downgrade(rt)));
        }
    })
}

/// The event loop the current task runs on, if this module scheduled on it.
fn current_loop() -> Option<Rc<LocalEventLoop>> {
    let id = Handle::try_current().ok()?.id();
    LOOPS.with(|loops| loops.borrow().iter().find(|(i, _)| *i == id)?.1.upgrade())
}

/// A task's hold on an event loop. A runtime cannot be dropped from inside
/// its own drive, so a release inside any runtime is deferred to a
/// microtask.
struct Hold(Option<Rc<LocalEventLoop>>);

impl Drop for Hold {
    fn drop(&mut self) {
        if let Some(rt) = self.0.take() {
            if Handle::try_current().is_ok() {
                crate::spawn_local(async move { drop(rt) });
            }
        }
    }
}

/// Spawns `future` as a root on `rt` and a sibling task that awaits its
/// `JoinHandle` and delivers the outcome to `on_complete`, so a root panic
/// arrives as `Err(JoinError)`. Both are queued only (the hosted loop
/// schedules its own drive); nothing runs on the caller's stack.
fn spawn_root<F, C>(rt: &LocalEventLoop, future: F, on_complete: C)
where
    F: Future + 'static,
    F::Output: 'static,
    C: FnOnce(Result<F::Output, JoinError>) + 'static,
{
    let root = rt.spawn_local(future);
    rt.spawn_local(async move { on_complete(root.await) });
}

/// Schedules `future` as a root on the ambient event loop, delivering its
/// outcome (or a panic, as `Err(JoinError)`) to `on_complete`, with the
/// drive semantics of [`schedule_on`].
pub fn schedule<F, C>(future: F, on_complete: C)
where
    F: Future + 'static,
    F::Output: 'static,
    C: FnOnce(Result<F::Output, JoinError>) + 'static,
{
    schedule_on(&ambient(), future, on_complete)
}

/// Schedules `future` as a root on `rt`, delivering its outcome (or a
/// panic, as `Err(JoinError)`) to `on_complete`. `rt` is the caller's, for
/// hosts that scope a loop to something longer-lived than one call, such as
/// one per Cloudflare Durable Object instance, each its own I/O context
/// (build it with [`new_event_loop`]).
///
/// Called outside any runtime context — a top-level JS call — this drives
/// one batch immediately, so the first poll is synchronous (parity with
/// `future_to_promise`). Called re-entrantly — a task's JS import invoking
/// an export mid-drive — driving on the caller's stack would nest the
/// runtime context, so the root is only queued: on `rt`'s own drive it is
/// picked up there, and from another loop's drive it is spawned from a
/// microtask once that drive returns (a hosted wake from inside one loop's
/// drive is folded into that loop's follow-up drive, so would be lost).
///
/// Nothing scheduled here holds `rt`, so the caller decides when the loop
/// goes: once the last holder lets go (each [`spawn_extending`] task is
/// one), it tears down as an isolated loop does. A runtime cannot be dropped
/// from inside its own drive, so let go of `rt` from a host callback or an
/// export, not from one of its own tasks.
pub fn schedule_on<F, C>(rt: &Rc<LocalEventLoop>, future: F, on_complete: C)
where
    F: Future + 'static,
    F::Output: 'static,
    C: FnOnce(Result<F::Output, JoinError>) + 'static,
{
    register(rt);
    match Handle::try_current() {
        Err(_) => {
            spawn_root(rt, future, on_complete);
            rt.drive();
        }
        Ok(current) if current.id() == rt.handle().id() => spawn_root(rt, future, on_complete),
        Ok(_) => {
            let hold = Hold(Some(rt.clone()));
            crate::spawn_local(async move {
                if let Some(rt) = &hold.0 {
                    spawn_root(rt, future, on_complete);
                }
            });
        }
    }
}

/// Schedules `future` as the root of a fresh event loop owned by this call,
/// with the same drive semantics as [`schedule`]. The event loop's reactor,
/// timers, and any tasks spawned inside `future` are fully isolated from
/// other invocations, for multiplexed hosts (e.g. Cloudflare Workers) where
/// one invocation's event loop must not perform I/O on behalf of another's
/// context.
///
/// The completion task holds the event loop, so it lives until the root
/// settles, or until the last task from [`spawn_extending`] finishes, and
/// then tears down with native `Runtime` drop semantics: spawned tasks still
/// in flight are dropped and the reactor is closed. The drop is deferred to
/// a microtask, since a runtime cannot be dropped from inside its own drive.
/// Armed host callbacks only hold weak references, so nothing else keeps it
/// alive.
pub fn schedule_isolated<F, C>(future: F, on_complete: C)
where
    F: Future + 'static,
    F::Output: 'static,
    C: FnOnce(Result<F::Output, JoinError>) + 'static,
{
    let rt = Rc::new(new_event_loop().expect("failed to build isolated tokio event loop"));
    let hold = Hold(Some(rt.clone()));
    schedule_on(&rt, future, move |out| {
        drop(hold);
        on_complete(out);
    });
}

/// Spawns `future` on the event loop the calling task runs on, holding the
/// loop alive until it finishes, for work that must outlive the root (a
/// Cloudflare Workers `ctx.waitUntil`): an isolated loop whose root settles
/// first is torn down only once it finishes, so the loop's other tasks run
/// on until then too. Errs `future` back outside a task on one of this
/// module's loops, as on the parked runtimes of [`block_on`].
///
/// The returned `JoinHandle` can be awaited from any executor, such as
/// `future_to_promise`; dropping it detaches the task, which keeps its hold.
/// A task that never finishes holds the loop for good.
pub fn spawn_extending<F>(future: F) -> Result<JoinHandle<F::Output>, F>
where
    F: Future + 'static,
    F::Output: 'static,
{
    let Some(rt) = current_loop() else {
        return Err(future);
    };
    let hold = Hold(Some(rt.clone()));
    Ok(rt.spawn_local(async move {
        let _hold = hold;
        future.await
    }))
}

/// Builds a parked runtime: a current-thread `Runtime` whose idle waits are
/// JSPI suspensions of the calling activation. Same driver fallback as
/// [`new_event_loop`].
fn build_parked() -> std::io::Result<Runtime> {
    match ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Err(e) if e.kind() == std::io::ErrorKind::Unsupported => {
            ::tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
        }
        built => built,
    }
}

/// Runs `future` to completion on the thread's ambient parked runtime, for
/// `#[wasm_bindgen(jspi, experimental_tokio)]`: the export is a promising
/// activation, and every wait of the runtime (timers, I/O, an idle scheduler)
/// is a JSPI suspension of that activation, which leaves the runtime. A
/// sibling invocation arriving meanwhile enters the same runtime and waits
/// on its own.
///
/// A suspension the runtime does not issue (`jspi_block_on_promise` or a
/// suspending import inside task code) parks the activation mid-poll. With
/// tokio's fiber-owned runtime context (`--cfg tokio_unstable_jspi_hooks`) a
/// sibling still enters, and its timers and I/O advance once the parked
/// activation resumes and releases the scheduler core; without it the
/// runtime stays entered across the suspension and the sibling's `block_on`
/// is a nested runtime.
pub fn block_on<F: Future>(future: F) -> F::Output {
    AMBIENT_PARKED.with(|cell| {
        cell.get_or_init(|| build_parked().expect("failed to build ambient tokio runtime"))
            .block_on(future)
    })
}

/// [`block_on`] on a fresh runtime owned by this call and dropped once
/// `future` settles (tasks still in flight are dropped, the reactor closed),
/// for `#[wasm_bindgen(jspi, experimental_tokio = "isolated")]`.
pub fn block_on_isolated<F: Future>(future: F) -> F::Output {
    build_parked()
        .expect("failed to build isolated tokio runtime")
        .block_on(future)
}
