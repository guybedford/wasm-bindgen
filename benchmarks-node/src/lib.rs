mod object_graph;
mod collections;
mod strings;
mod buffers;
mod async_ops;
mod marshalling;
mod closures;
mod errors;

use wasm_bindgen::prelude::*;

/// Benchmark 1: Object Graph Construction & Traversal
/// Tests structural typing, property access, and object creation patterns
#[wasm_bindgen]
pub fn bench_object_graph() {
    object_graph::run();
}

/// Benchmark 2: Collection Operations with Closures
/// Tests closure creation/invocation and iterator overhead with JS arrays
#[wasm_bindgen]
pub fn bench_collections() {
    collections::run();
}

/// Benchmark 3: String Processing Workloads
/// Tests string marshalling at various sizes and encodings
#[wasm_bindgen]
pub fn bench_strings() {
    strings::run();
}

/// Benchmark 4: TypedArray/Buffer Operations
/// Tests zero-copy transfer and typed array manipulation
#[wasm_bindgen]
pub fn bench_buffers() {
    buffers::run();
}

/// Benchmark 5: Async/Promise Patterns
/// Tests Future→Promise overhead and async runtime cost
#[wasm_bindgen]
pub async fn bench_async() {
    async_ops::run().await;
}

/// Benchmark 6: Complex Type Marshalling
/// Tests serde-wasm-bindgen and JsValue conversion overhead
#[wasm_bindgen]
pub fn bench_marshalling() {
    marshalling::run();
}

/// Benchmark 7: Closure Lifecycle & Event Patterns
/// Tests Closure::new(), long-lived closures, and memory management
#[wasm_bindgen]
pub fn bench_closures() {
    closures::run();
}

/// Benchmark 8: Error Handling Paths
/// Tests Result<T,E> and exception propagation overhead
#[wasm_bindgen]
pub fn bench_errors() -> Result<(), JsValue> {
    errors::run()
}
