use wasm_bindgen::prelude::*;
use js_sys::Array;

/// Benchmark: Collection Operations
///
/// Simulates data processing pipelines with arrays and iterations.
/// Common pattern in data-heavy applications.
///
/// Tests:
/// - Array creation and manipulation
/// - Element access and pushing
/// - Type conversions (numbers, booleans)
/// - Multiple passes over data
pub fn run() {
    // Create a large array of numbers
    let arr = create_number_array(10_000);

    // Map: square each number
    let mapped = map_square(&arr);
    assert_eq!(mapped.length(), 10_000);

    // Filter: keep only even numbers
    let filtered = filter_even(&mapped);
    assert!(filtered.length() > 0);

    // Reduce: sum all numbers
    let sum = reduce_sum(&filtered);
    assert!(sum > 0.0);

    // Chain operations: realistic data pipeline
    let result = pipeline(&arr);
    assert!(result > 0.0);
}

/// Create a JS array of numbers
fn create_number_array(size: u32) -> Array {
    let arr = Array::new();
    for i in 0..size {
        arr.push(&JsValue::from(i));
    }
    arr
}

/// Map operation: square each number
fn map_square(arr: &Array) -> Array {
    let result = Array::new();
    let len = arr.length();

    for i in 0..len {
        let elem = arr.get(i);
        if let Some(num) = elem.as_f64() {
            result.push(&JsValue::from(num * num));
        } else {
            result.push(&elem);
        }
    }

    result
}

/// Filter operation: keep only even numbers
fn filter_even(arr: &Array) -> Array {
    let result = Array::new();
    let len = arr.length();

    for i in 0..len {
        let elem = arr.get(i);
        if let Some(num) = elem.as_f64() {
            if num as u64 % 2 == 0 {
                result.push(&elem);
            }
        }
    }

    result
}

/// Reduce operation: sum all numbers
fn reduce_sum(arr: &Array) -> f64 {
    let len = arr.length();
    let mut sum = 0.0;

    for i in 0..len {
        let elem = arr.get(i);
        if let Some(num) = elem.as_f64() {
            sum += num;
        }
    }

    sum
}

/// Complex pipeline: map → filter → reduce
fn pipeline(arr: &Array) -> f64 {
    let len = arr.length();

    // Step 1: Map (multiply by 2)
    let mapped = Array::new();
    for i in 0..len {
        let elem = arr.get(i);
        if let Some(num) = elem.as_f64() {
            mapped.push(&JsValue::from(num * 2.0));
        }
    }

    // Step 2: Filter (only numbers > 5000)
    let filtered = Array::new();
    let mapped_len = mapped.length();
    for i in 0..mapped_len {
        let elem = mapped.get(i);
        if let Some(num) = elem.as_f64() {
            if num > 5000.0 {
                filtered.push(&elem);
            }
        }
    }

    // Step 3: Reduce (sum)
    let mut sum = 0.0;
    let filtered_len = filtered.length();
    for i in 0..filtered_len {
        let elem = filtered.get(i);
        if let Some(num) = elem.as_f64() {
            sum += num;
        }
    }

    sum
}
