use wasm_bindgen::prelude::*;

/// Benchmark: Error Handling Paths
///
/// Tests Result<T,E> patterns and exception handling overhead across
/// the Rust/JS boundary. Simulates robust error handling in real applications.
///
/// Tests:
/// - Result<T, JsValue> overhead
/// - Exception throwing and catching
/// - Error propagation through call stack
/// - Mixed success/error paths
/// - Error message construction
pub fn run() -> Result<(), JsValue> {
    // Test 1: Many operations with 10% error rate (5000 ops)
    let (success, errors) = mixed_result_operations(5_000, 10);
    assert!(success > 0);
    assert!(errors > 0);

    // Test 2: Error propagation through stack (1000 iterations)
    for i in 0..1_000 {
        let result = error_propagation(i % 10 == 0);
        if i % 10 == 0 {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
    }

    // Test 3: Try/catch overhead (10,000 operations)
    try_catch_overhead(10_000)?;

    // Test 4: Complex error types (1000 operations)
    complex_error_handling(1_000)?;

    Ok(())
}

/// Perform operations that succeed or fail based on error_rate percentage
fn mixed_result_operations(count: u32, error_rate: u32) -> (u32, u32) {
    let mut success_count = 0;
    let mut error_count = 0;

    for i in 0..count {
        let result = if i % 100 < error_rate {
            Err(format!("Error at iteration {}", i))
        } else {
            Ok(i * 2)
        };

        match result {
            Ok(value) => {
                success_count += 1;
                // Use the value to prevent optimization
                assert!(value >= 0);
            }
            Err(_msg) => {
                error_count += 1;
            }
        }
    }

    (success_count, error_count)
}

/// Test error propagation through call stack
fn error_propagation(should_error: bool) -> Result<u32, String> {
    level_1(should_error)
}

fn level_1(should_error: bool) -> Result<u32, String> {
    level_2(should_error)
}

fn level_2(should_error: bool) -> Result<u32, String> {
    level_3(should_error)
}

fn level_3(should_error: bool) -> Result<u32, String> {
    if should_error {
        Err("Error in level 3".to_string())
    } else {
        Ok(42)
    }
}

/// Test try/catch overhead with Result types
fn try_catch_overhead(count: u32) -> Result<(), JsValue> {
    for i in 0..count {
        // This simulates operations that might fail
        let result = risky_operation(i);

        // Handle the result (simulating try/catch pattern)
        match result {
            Ok(value) => {
                // Process success
                assert!(value > 0);
            }
            Err(e) => {
                // Handle error - convert to JsValue
                return Err(JsValue::from_str(&e));
            }
        }
    }

    Ok(())
}

fn risky_operation(value: u32) -> Result<u32, String> {
    // Never actually errors in this benchmark (testing happy path overhead)
    if value == u32::MAX {
        Err("Overflow".to_string())
    } else {
        Ok(value + 1)
    }
}

/// Complex error handling with multiple error types
fn complex_error_handling(count: u32) -> Result<(), JsValue> {
    for i in 0..count {
        // Operation that can fail in different ways
        let result = match i % 5 {
            0 => parse_number(&format!("{}", i)),
            1 => validate_range(i, 0, 10_000),
            2 => process_data(&format!("data_{}", i)),
            3 => check_state(i),
            _ => Ok(i),
        };

        // Propagate errors
        result?;
    }

    Ok(())
}

fn parse_number(s: &str) -> Result<u32, JsValue> {
    s.parse::<u32>()
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
}

fn validate_range(value: u32, min: u32, max: u32) -> Result<u32, JsValue> {
    if value >= min && value <= max {
        Ok(value)
    } else {
        Err(JsValue::from_str(&format!(
            "Value {} out of range [{}, {}]",
            value, min, max
        )))
    }
}

fn process_data(data: &str) -> Result<u32, JsValue> {
    if data.is_empty() {
        Err(JsValue::from_str("Empty data"))
    } else {
        Ok(data.len() as u32)
    }
}

fn check_state(value: u32) -> Result<u32, JsValue> {
    if value % 2 == 0 {
        Ok(value)
    } else {
        Ok(value + 1)
    }
}

/// Export a function that can throw an error (for testing from JS)
#[wasm_bindgen]
pub fn may_throw(should_throw: bool) -> Result<u32, JsValue> {
    if should_throw {
        Err(JsValue::from_str("Intentional error for testing"))
    } else {
        Ok(42)
    }
}

/// Export a function that always succeeds (baseline)
#[wasm_bindgen]
pub fn never_throws() -> Result<u32, JsValue> {
    Ok(42)
}
