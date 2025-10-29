use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::Promise;

/// Benchmark: Async/Promise Patterns
///
/// Simulates async operations using Promises and Rust futures.
/// Common pattern in API calls, file I/O, and async data processing.
///
/// Tests:
/// - Promise creation overhead
/// - Future ↔ Promise conversion
/// - Promise chaining
/// - Parallel promise execution
/// - Async/await overhead
pub async fn run() {
    // Test 1: Sequential promise chain (1000 operations)
    let result = sequential_promises(1000).await;
    // Sum of 0..1000 = 1000 * 999 / 2 = 499500
    assert_eq!(result, 499500);

    // Test 2: Parallel promises (100 concurrent tasks)
    let results = parallel_promises(100).await;
    // Sum of 0..100 = 100 * 99 / 2 = 4950
    assert_eq!(results, 4950);

    // Test 3: Nested async calls
    let nested_result = nested_async_calls(50).await;
    assert!(nested_result > 0);
}

/// Sequential promise chain - each awaits the previous
async fn sequential_promises(count: u32) -> u32 {
    let mut total = 0;

    for i in 0..count {
        let value = simulate_async_operation(i).await;
        total += value;
    }

    total
}

/// Parallel promises - launch all at once, await all results
async fn parallel_promises(count: u32) -> u32 {
    // Create all promises
    let mut futures = Vec::new();

    for i in 0..count {
        let future = simulate_async_operation(i);
        futures.push(future);
    }

    // Await all (simulating Promise.all)
    let mut total = 0;
    for future in futures {
        total += future.await;
    }

    total
}

/// Nested async calls - each level calls the next
fn nested_async_calls(depth: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = u32>>> {
    Box::pin(async move {
        if depth == 0 {
            1
        } else {
            let inner = nested_async_calls(depth - 1).await;
            simulate_async_operation(inner).await
        }
    })
}

/// Simulate an async operation that resolves to a value
/// Uses Promise::resolve() to create an immediately-resolved promise
async fn simulate_async_operation(value: u32) -> u32 {
    let promise = Promise::resolve(&JsValue::from(value));
    let future = JsFuture::from(promise);

    match future.await {
        Ok(val) => val.as_f64().unwrap_or(0.0) as u32,
        Err(_) => 0,
    }
}

/// Create a promise that performs some computation before resolving
#[wasm_bindgen]
pub fn create_computational_promise(iterations: u32) -> Promise {
    let promise = Promise::new(&mut |resolve, _reject| {
        // Do some work
        let mut sum = 0u32;
        for i in 0..iterations {
            sum = sum.wrapping_add(i);
        }

        // Resolve with result
        let _ = resolve.call1(&JsValue::NULL, &JsValue::from(sum));
    });

    promise
}

/// Chain multiple promise operations
#[wasm_bindgen]
pub async fn promise_chain(start_value: u32) -> u32 {
    // Chain several operations
    let promise1 = Promise::resolve(&JsValue::from(start_value));
    let value1 = JsFuture::from(promise1).await.ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;

    let promise2 = Promise::resolve(&JsValue::from(value1 * 2));
    let value2 = JsFuture::from(promise2).await.ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;

    let promise3 = Promise::resolve(&JsValue::from(value2 + 10));
    let value3 = JsFuture::from(promise3).await.ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;

    value3
}
