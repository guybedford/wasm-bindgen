use wasm_bindgen::prelude::*;
use js_sys::{Object, Reflect};

/// Benchmark: Object Graph Construction & Traversal
///
/// Simulates building a component tree (like a virtual DOM or UI hierarchy)
/// with 1000+ nested objects, then traversing to compute aggregate properties.
///
/// Tests:
/// - Object creation (Object::new())
/// - Property setting (Reflect::set)
/// - Property getting (Reflect::get)
/// - Structural typing overhead
/// - Object graph traversal patterns
pub fn run() {
    // Build a tree: 10 levels deep, 3 children per node = ~88,000 nodes
    let root = build_tree(8, 3);

    // Traverse and count nodes
    let count = count_nodes(&root);

    // Traverse and sum all "value" properties
    let sum = sum_values(&root);

    // Prevent optimization from eliminating the work
    assert!(count > 1000);
    assert!(sum > 0);
}

/// Build a tree of JS objects recursively
fn build_tree(depth: u32, children: u32) -> Object {
    let obj = Object::new();

    // Set some properties
    let _ = Reflect::set(&obj, &"depth".into(), &depth.into());
    let _ = Reflect::set(&obj, &"value".into(), &(depth * 10).into());
    let _ = Reflect::set(&obj, &"name".into(), &format!("Node-{}", depth).into());

    if depth > 0 {
        let children_array = js_sys::Array::new();

        for i in 0..children {
            let child = build_tree(depth - 1, children);
            let _ = Reflect::set(&child, &"childIndex".into(), &i.into());
            children_array.push(&child);
        }

        let _ = Reflect::set(&obj, &"children".into(), &children_array);
    }

    obj
}

/// Recursively count all nodes in the tree
fn count_nodes(obj: &Object) -> u32 {
    let mut count = 1;

    // Try to get children array
    if let Ok(children_val) = Reflect::get(obj, &"children".into()) {
        if let Ok(children_array) = children_val.dyn_into::<js_sys::Array>() {
            for i in 0..children_array.length() {
                if let Some(child_obj) = children_array.get(i).dyn_ref::<Object>() {
                    count += count_nodes(child_obj);
                }
            }
        }
    }

    count
}

/// Recursively sum all "value" properties
fn sum_values(obj: &Object) -> u32 {
    let mut sum = 0;

    // Get this node's value
    if let Ok(value_val) = Reflect::get(obj, &"value".into()) {
        if let Some(num) = value_val.as_f64() {
            sum += num as u32;
        }
    }

    // Recurse to children
    if let Ok(children_val) = Reflect::get(obj, &"children".into()) {
        if let Ok(children_array) = children_val.dyn_into::<js_sys::Array>() {
            for i in 0..children_array.length() {
                if let Some(child_obj) = children_array.get(i).dyn_ref::<Object>() {
                    sum += sum_values(child_obj);
                }
            }
        }
    }

    sum
}
