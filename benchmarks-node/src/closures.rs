use wasm_bindgen::prelude::*;
use js_sys::Function;

/// Benchmark: Closure Lifecycle & Event Patterns
///
/// Tests closure creation, invocation, and cleanup patterns.
/// Simulates event handlers and callback-heavy code without actual DOM events.
///
/// Tests:
/// - Closure::new() overhead
/// - Closure invocation from JS
/// - Closure::forget() and memory management
/// - Short-lived vs long-lived closures
/// - Closure with captured state
pub fn run() {
    // Test 1: Create and invoke many short-lived closures (10,000)
    short_lived_closures(10_000);

    // Test 2: Simulate event handler pattern (1000 handlers, 10 invocations each)
    event_handler_pattern(1_000, 10);

    // Test 3: Closures with captured state
    stateful_closures(5_000);

    // Test 4: Function composition with closures
    closure_composition(1_000);
}

/// Create, invoke, and drop many closures
fn short_lived_closures(count: u32) {
    for i in 0..count {
        let closure = Closure::wrap(Box::new(move |x: u32| -> u32 {
            x + i
        }) as Box<dyn Fn(u32) -> u32>);

        // Invoke the closure
        let func = closure.as_ref().unchecked_ref::<Function>();
        let result = func.call1(&JsValue::NULL, &JsValue::from(10));

        // Verify
        if let Ok(val) = result {
            if let Some(num) = val.as_f64() {
                assert!(num >= 10.0);
            }
        }

        // Closure is dropped here (not forgotten)
        drop(closure);
    }
}

/// Simulate event handler pattern: create handlers, invoke multiple times
fn event_handler_pattern(handler_count: u32, invocations_per_handler: u32) {
    let mut handlers = Vec::new();

    // Create handlers (would normally attach to elements)
    for i in 0..handler_count {
        let handler = Closure::wrap(Box::new(move |event_data: u32| -> u32 {
            // Simulate event processing
            event_data + i * 100
        }) as Box<dyn Fn(u32) -> u32>);

        handlers.push(handler);
    }

    // Invoke each handler multiple times (simulating events firing)
    for handler in &handlers {
        let func = handler.as_ref().unchecked_ref::<Function>();

        for j in 0..invocations_per_handler {
            let _ = func.call1(&JsValue::NULL, &JsValue::from(j));
        }
    }

    // Clean up: in real code these would be long-lived and forgotten
    // Here we let them drop naturally
}

/// Closures that capture and modify state
fn stateful_closures(count: u32) {
    for i in 0..count {
        // Create a closure that captures local state
        let base_value = i;

        let closure = Closure::wrap(Box::new(move |multiplier: u32| -> u32 {
            base_value * multiplier
        }) as Box<dyn Fn(u32) -> u32>);

        // Invoke with different arguments
        let func = closure.as_ref().unchecked_ref::<Function>();

        let result1 = func.call1(&JsValue::NULL, &JsValue::from(2));
        let result2 = func.call1(&JsValue::NULL, &JsValue::from(3));
        let result3 = func.call1(&JsValue::NULL, &JsValue::from(5));

        // Verify results
        if let (Ok(r1), Ok(r2), Ok(r3)) = (result1, result2, result3) {
            if let (Some(n1), Some(n2), Some(n3)) = (
                r1.as_f64(),
                r2.as_f64(),
                r3.as_f64(),
            ) {
                assert_eq!(n1 as u32, base_value * 2);
                assert_eq!(n2 as u32, base_value * 3);
                assert_eq!(n3 as u32, base_value * 5);
            }
        }

        drop(closure);
    }
}

/// Function composition: create closures that call other closures
fn closure_composition(count: u32) {
    for i in 0..count {
        // Inner closure: adds i
        let add_closure = Closure::wrap(Box::new(move |x: u32| -> u32 {
            x + i
        }) as Box<dyn Fn(u32) -> u32>);

        let add_func = add_closure.as_ref().unchecked_ref::<Function>();

        // Middle closure: multiplies by 2
        let mul_closure = Closure::wrap(Box::new(move |x: u32| -> u32 {
            x * 2
        }) as Box<dyn Fn(u32) -> u32>);

        let mul_func = mul_closure.as_ref().unchecked_ref::<Function>();

        // Compose: (x + i) * 2
        let input = 10u32;
        let added = add_func.call1(&JsValue::NULL, &JsValue::from(input))
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;

        let result = mul_func.call1(&JsValue::NULL, &JsValue::from(added))
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;

        assert_eq!(result, (input + i) * 2);

        drop(add_closure);
        drop(mul_closure);
    }
}

/// Export a function that creates a long-lived closure (for testing forget pattern)
#[wasm_bindgen]
pub fn create_callback(value: u32) -> Function {
    let closure = Closure::wrap(Box::new(move |x: u32| -> u32 {
        x + value
    }) as Box<dyn Fn(u32) -> u32>);

    let func = closure.as_ref().unchecked_ref::<Function>().clone();

    // Forget the closure so it lives forever (typical for event handlers)
    closure.forget();

    func
}
