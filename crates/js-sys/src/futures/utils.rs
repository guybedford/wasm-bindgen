//! Promise combinators that delegate to JavaScript's `Promise.all`, `Promise.race`, etc.
//!
//! These provide true concurrent I/O by using the JavaScript event loop rather
//! than cooperative Rust polling. Use these instead of `futures_util::future::join_all`
//! when working with JS-backed async operations (fetch, KV, D1, R2, etc).

use core::future::Future;

use wasm_bindgen::convert::{FromWasmAbi, Upcast};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsGeneric;

use crate::Promise;

use super::future_to_promise_typed;

/// Trait for types that can be converted into a JavaScript `Promise<T>`.
///
/// Implemented for [`Promise<T>`] (identity conversion) and for Rust `Future`s
/// with output `Result<T, JsValue>` (via [`future_to_promise_typed`]).
pub trait IntoPromise {
    /// The type this promise resolves to.
    type Output;

    /// Convert this value into a JavaScript [`Promise`].
    fn into_promise(self) -> Promise<Self::Output>;
}

impl<T: JsGeneric> IntoPromise for Promise<T> {
    type Output = T;

    fn into_promise(self) -> Promise<T> {
        self
    }
}

impl<F, T> IntoPromise for F
where
    F: Future<Output = Result<T, JsValue>> + 'static,
    T: FromWasmAbi + JsGeneric + Upcast<T> + 'static,
{
    type Output = T;

    fn into_promise(self) -> Promise<T> {
        future_to_promise_typed(self)
    }
}

/// Collects an iterator of `IntoPromise` items into a typed `Array<Promise<T>>`.
fn collect_promises<T, I>(promises: I) -> crate::Array<Promise<T>>
where
    T: JsGeneric,
    I: IntoIterator,
    I::Item: IntoPromise<Output = T>,
{
    let array = crate::Array::<Promise<T>>::new_typed();
    for p in promises {
        array.push(&p.into_promise());
    }
    array
}

/// Awaits multiple JavaScript `Promise`s concurrently using `Promise.all`.
///
/// Unlike `futures_util::future::join_all`, which polls futures cooperatively
/// within the Rust executor and cannot yield to the JavaScript event loop
/// between individual completions, this function delegates concurrency to
/// `Promise.all` on the JavaScript side. This enables true concurrent I/O for
/// JS-backed operations such as `fetch`, KV, D1, R2, etc.
///
/// All promises must resolve to the same type `T`. For heterogeneous promise
/// types, use the [`join!`] macro instead.
///
/// Accepts any iterator of items that implement [`IntoPromise`], which includes
/// both [`Promise<T>`] values and Rust `Future`s whose output is
/// `Result<T, JsValue>`.
///
/// # Example
///
/// ```ignore
/// use js_sys::futures::join_all;
///
/// let promises: Vec<Promise> = (0..10)
///     .map(|_| worker.fetch_with_str_and_init(&url, &init))
///     .collect();
/// let results: Array = join_all(promises).await?;
/// ```
///
/// # Errors
///
/// Rejects with the value of the first promise that rejects, mirroring the
/// behavior of `Promise.all`.
pub async fn join_all<T, I>(promises: I) -> Result<crate::Array<T>, JsValue>
where
    T: JsGeneric + FromWasmAbi + 'static,
    I: IntoIterator,
    I::Item: IntoPromise<Output = T>,
{
    Promise::all_iterable(&collect_promises(promises)).await
}

/// Awaits multiple JavaScript `Promise`s concurrently using `Promise.allSettled`.
///
/// Unlike [`join_all`], this never rejects early. It waits for every promise to
/// either fulfill or reject, returning an `Array<PromiseState<T>>` where each
/// element can be inspected via `.is_fulfilled()`, `.get_value()`, and
/// `.get_reason()`.
///
/// For heterogeneous promise types, use the [`all_settled!`] macro instead.
///
/// # Example
///
/// ```ignore
/// use js_sys::futures::all_settled;
///
/// let results = all_settled(promises).await?;
/// for state in results.iter() {
///     if state.is_fulfilled() {
///         let value = state.get_value().unwrap();
///     }
/// }
/// ```
pub async fn all_settled<T, I>(promises: I) -> Result<crate::Array<crate::PromiseState<T>>, JsValue>
where
    T: JsGeneric + FromWasmAbi + 'static,
    I: IntoIterator,
    I::Item: IntoPromise<Output = T>,
{
    Promise::all_settled_iterable(&collect_promises(promises)).await
}

/// Returns the result of the first `Promise` to settle (fulfill or reject),
/// using `Promise.race`.
///
/// This is the JS-native equivalent of `futures_util::future::select`. All
/// promises must resolve to the same type `T`.
///
/// # Example
///
/// ```ignore
/// use js_sys::futures::race;
///
/// let first = race(promises).await?;
/// ```
///
/// # Errors
///
/// Rejects with the value of the first promise to reject, if it settles
/// before any promise fulfills.
pub async fn race<T, I>(promises: I) -> Result<T, JsValue>
where
    T: JsGeneric + FromWasmAbi + 'static,
    I: IntoIterator,
    I::Item: IntoPromise<Output = T>,
{
    Promise::race_iterable(&collect_promises(promises)).await
}

/// Returns the result of the first `Promise` to fulfill, using `Promise.any`.
///
/// Ignores rejections unless all promises reject, in which case it rejects
/// with an `AggregateError`.
///
/// # Example
///
/// ```ignore
/// use js_sys::futures::any;
///
/// let first_success = any(promises).await?;
/// ```
///
/// # Errors
///
/// Rejects with an `AggregateError` if every promise in the iterator rejects.
pub async fn any<T, I>(promises: I) -> Result<T, JsValue>
where
    T: JsGeneric + FromWasmAbi + 'static,
    I: IntoIterator,
    I::Item: IntoPromise<Output = T>,
{
    Promise::any_iterable(&collect_promises(promises)).await
}

/// Maps a tuple of `Promising` types to tuples of their resolution types.
///
/// For example, `(Promise<A>, Promise<B>): PromiseTuple` has
/// `Resolved = (A, B)` and `Settled = (PromiseState<A>, PromiseState<B>)`.
pub trait PromiseTuple: crate::JsTuple {
    /// The tuple of resolved types, for `Promise.all`.
    type Resolved: crate::JsTuple;
    /// The tuple of settled types, for `Promise.allSettled`.
    type Settled: crate::JsTuple;
}

macro_rules! impl_promise_tuple {
    ($($T:ident),+) => {
        impl<$($T: crate::Promising),+> PromiseTuple for ($($T,)+) {
            type Resolved = ($($T::Resolution,)+);
            type Settled = ($(crate::PromiseState<$T::Resolution>,)+);
        }
    };
}

impl_promise_tuple!(T1);
impl_promise_tuple!(T1, T2);
impl_promise_tuple!(T1, T2, T3);
impl_promise_tuple!(T1, T2, T3, T4);
impl_promise_tuple!(T1, T2, T3, T4, T5);
impl_promise_tuple!(T1, T2, T3, T4, T5, T6);
impl_promise_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_promise_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

impl<T: PromiseTuple> crate::ArrayTuple<T> {
    /// Concurrently awaits all promises in this tuple using `Promise.all`.
    ///
    /// Returns a `Promise` that resolves to an `ArrayTuple` of the resolved
    /// types. Use `.into_parts()` on the result to destructure into a Rust
    /// tuple.
    pub fn promise_all(&self) -> Promise<crate::ArrayTuple<T::Resolved>> {
        use wasm_bindgen::JsCast as _;
        Promise::all_iterable(self).unchecked_into()
    }

    /// Concurrently settles all promises in this tuple using
    /// `Promise.allSettled`. Never rejects early.
    pub fn promise_all_settled(&self) -> Promise<crate::ArrayTuple<T::Settled>> {
        use wasm_bindgen::JsCast as _;
        Promise::all_settled_iterable(self).unchecked_into()
    }
}

/// Awaits multiple JavaScript `Promise`s of different types concurrently using
/// `Promise.all`, returning an `ArrayTuple` of results.
///
/// This is the heterogeneous counterpart to [`join_all`]. Each argument must
/// be a `Promise<T>`. The result is a `Promise<ArrayTuple<(T1, T2, ...)>>`
/// which can be `.await`ed and then destructured via `.into_parts()`.
///
/// # Example
///
/// ```ignore
/// use js_sys::join;
///
/// let results = join!(
///     fetch_promise,        // Promise<Response>
///     array_buffer_promise, // Promise<ArrayBuffer>
/// ).await?;
/// let (response, buffer) = results.into_parts();
/// ```
///
/// # Errors
///
/// Returns `Err(JsValue)` if any promise rejects, with the value of the first
/// rejection.
#[macro_export]
macro_rules! join {
    ($($promise:expr),+ $(,)?) => {{
        let promises: $crate::ArrayTuple<_> = ($($promise,)+).into();
        promises.promise_all()
    }};
}

/// Awaits multiple JavaScript `Promise`s of different types concurrently using
/// `Promise.allSettled`, returning an `ArrayTuple` of `PromiseState` results.
///
/// This is the heterogeneous counterpart to [`all_settled`]. Each argument must
/// be a `Promise<T>`. Unlike [`join!`], this never rejects early — it waits for
/// every promise to settle.
///
/// # Example
///
/// ```ignore
/// use js_sys::all_settled;
///
/// let results = all_settled!(
///     fetch_promise,        // Promise<Response>
///     array_buffer_promise, // Promise<ArrayBuffer>
/// ).await?;
/// ```
#[macro_export]
macro_rules! all_settled {
    ($($promise:expr),+ $(,)?) => {{
        let promises: $crate::ArrayTuple<_> = ($($promise,)+).into();
        promises.promise_all_settled()
    }};
}
