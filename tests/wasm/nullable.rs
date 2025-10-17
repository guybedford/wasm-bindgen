use js_sys::{JsString, Number, Object};
use wasm_bindgen::convert::Upcast;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Null, Nullable, Undefined};
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/nullable.js")]
extern "C" {
    fn return_null() -> Nullable<Number>;
    fn return_undefined() -> Nullable<Number>;
    fn return_number() -> Nullable<Number>;
    fn return_string() -> Nullable<JsString>;

    fn take_nullable_null(val: Nullable<Number>);
    fn take_nullable_value(val: Nullable<Number>);
    fn take_nullable_number(val: Nullable<Number>);
    fn take_nullable_string(val: Nullable<JsString>);

    fn test_nullable_exports();
}

#[wasm_bindgen_test]
fn test_new() {
    let empty: Nullable<Number> = Nullable::new();
    assert!(empty.is_empty());
}

#[wasm_bindgen_test]
fn test_wrap() {
    let num = Nullable::wrap(Number::from(42));
    assert!(!num.is_empty());
}

#[wasm_bindgen_test]
fn test_from_option_some() {
    let opt = Some(Number::from(123));
    let nullable = Nullable::from_option(opt);
    assert!(!nullable.is_empty());
    assert_eq!(nullable.unwrap().value_of(), 123.0);
}

#[wasm_bindgen_test]
fn test_from_option_none() {
    let opt: Option<Number> = None;
    let nullable = Nullable::from_option(opt);
    assert!(nullable.is_empty());
}

#[wasm_bindgen_test]
fn test_is_empty_null() {
    let val = return_null();
    assert!(val.is_empty());
}

#[wasm_bindgen_test]
fn test_is_empty_undefined() {
    let val = return_undefined();
    assert!(val.is_empty());
}

#[wasm_bindgen_test]
fn test_is_empty_value() {
    let val = return_number();
    assert!(!val.is_empty());
}

#[wasm_bindgen_test]
fn test_as_option_some() {
    let val = return_number();
    let opt = val.as_option();
    assert!(opt.is_some());
    assert_eq!(opt.unwrap().value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_as_option_none() {
    let val = return_null();
    let opt = val.as_option();
    assert!(opt.is_none());
}

#[wasm_bindgen_test]
fn test_into_option_some() {
    let val = return_number();
    let opt = val.into_option();
    assert!(opt.is_some());
    assert_eq!(opt.unwrap().value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_into_option_none() {
    let val = return_null();
    let opt = val.into_option();
    assert!(opt.is_none());
}

#[wasm_bindgen_test]
fn test_unwrap_success() {
    let val = return_number();
    let num = val.unwrap();
    assert_eq!(num.value_of(), 42.0);
}

#[wasm_bindgen_test]
#[should_panic(expected = "called `Nullable::unwrap()` on an empty value")]
fn test_unwrap_panic() {
    let val = return_null();
    val.unwrap();
}

#[wasm_bindgen_test]
fn test_expect_success() {
    let val = return_number();
    let num = val.expect("should have value");
    assert_eq!(num.value_of(), 42.0);
}

#[wasm_bindgen_test]
#[should_panic(expected = "custom error message")]
fn test_expect_panic() {
    let val = return_null();
    val.expect("custom error message");
}

#[wasm_bindgen_test]
fn test_unwrap_or_default() {
    let val = return_null();
    let num = val.unwrap_or_default();
    // Number::default() is Number::from(0)
    assert_eq!(num.value_of(), 0.0);

    let val = return_number();
    let num = val.unwrap_or_default();
    assert_eq!(num.value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_unwrap_or_else() {
    let val = return_null();
    let num = val.unwrap_or_else(|| Number::from(99));
    assert_eq!(num.value_of(), 99.0);

    let val = return_number();
    let num = val.unwrap_or_else(|| Number::from(99));
    assert_eq!(num.value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_import_null() {
    let val = return_null();
    assert!(val.is_empty());
}

#[wasm_bindgen_test]
fn test_import_undefined() {
    let val = return_undefined();
    assert!(val.is_empty());
}

#[wasm_bindgen_test]
fn test_import_value() {
    let val = return_number();
    assert!(!val.is_empty());
    assert_eq!(val.unwrap().value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_import_string() {
    let val = return_string();
    assert!(!val.is_empty());
    assert_eq!(val.unwrap(), "hello");
}

#[wasm_bindgen_test]
fn test_export_null() {
    take_nullable_null(Nullable::new());
}

#[wasm_bindgen_test]
fn test_export_value() {
    take_nullable_value(Nullable::wrap(Number::from(123)));
}

#[wasm_bindgen_test]
fn test_js_calls_rust() {
    test_nullable_exports();
}

// Exported functions for JS to call
#[wasm_bindgen]
pub fn rust_return_nullable_null() -> Nullable<Number> {
    Nullable::new()
}

#[wasm_bindgen]
pub fn rust_return_nullable_value() -> Nullable<Number> {
    Nullable::wrap(Number::from(456))
}

#[wasm_bindgen]
pub fn rust_take_nullable_null(val: Nullable<Number>) {
    assert!(val.is_empty());
}

#[wasm_bindgen]
pub fn rust_take_nullable_value(val: Nullable<Number>) {
    assert!(!val.is_empty());
    assert_eq!(val.unwrap().value_of(), 789.0);
}

#[wasm_bindgen_test]
fn test_debug_value() {
    let val = Nullable::wrap(Number::from(42));
    let debug_str = format!("{:?}", val);
    assert!(debug_str.contains("Number"));
    assert!(debug_str.contains("42"));
}

#[wasm_bindgen_test]
fn test_debug_null() {
    let val: Nullable<Number> = Nullable::new();
    let debug_str = format!("{:?}", val);
    assert!(debug_str.contains("Number"));
    assert!(debug_str.contains("null"));
}

#[wasm_bindgen_test]
fn test_default() {
    let val: Nullable<Number> = Default::default();
    assert!(val.is_empty());
}

#[wasm_bindgen_test]
fn test_nullable_in_generic_context() {
    fn process<T: wasm_bindgen::convert::JsGeneric>(nullable: Nullable<T>) -> bool {
        nullable.is_empty()
    }

    let empty: Nullable<Number> = Nullable::new();
    assert!(process(empty));

    let filled = Nullable::wrap(Number::from(1));
    assert!(!process(filled));
}

// ============================================================================
// Upcast tests
// ============================================================================

#[wasm_bindgen_test]
fn test_upcast_value_to_nullable() {
    // A Number can upcast to Nullable<Number>
    let num = Number::from(42);
    let nullable: Nullable<Number> = num.upcast();
    assert!(!nullable.is_empty());
    assert_eq!(nullable.unwrap().value_of(), 42.0);
}

#[wasm_bindgen_test]
fn test_upcast_string_to_nullable() {
    // A JsString can upcast to Nullable<JsString>
    let s = JsString::from("hello");
    let nullable: Nullable<JsString> = s.upcast();
    assert!(!nullable.is_empty());
    assert_eq!(nullable.unwrap(), "hello");
}

#[wasm_bindgen_test]
fn test_upcast_null_to_nullable() {
    // Null can upcast to Nullable<T> for any T
    let null = Null::NULL;
    let nullable: Nullable<Number> = null.upcast();
    assert!(nullable.is_empty());
}

#[wasm_bindgen_test]
fn test_upcast_undefined_to_nullable() {
    // Undefined can upcast to Nullable<T> for any T
    let undef = Undefined::UNDEFINED;
    let nullable: Nullable<Number> = undef.upcast();
    assert!(nullable.is_empty());
}

#[wasm_bindgen_test]
fn test_upcast_null_to_different_nullable_types() {
    // Null upcasts to Nullable of any type
    let null = Null::NULL;

    let nullable_num: Nullable<Number> = null.upcast();
    assert!(nullable_num.is_empty());

    let null = Null::NULL;
    let nullable_str: Nullable<JsString> = null.upcast();
    assert!(nullable_str.is_empty());

    let null = Null::NULL;
    let nullable_obj: Nullable<Object> = null.upcast();
    assert!(nullable_obj.is_empty());
}

#[wasm_bindgen_test]
fn test_upcast_in_function_call() {
    // Test using upcast to pass a value to a function expecting Nullable
    let num = Number::from(123);
    take_nullable_number(num.upcast());

    let s = JsString::from("test");
    take_nullable_string(s.upcast());
}

#[wasm_bindgen_test]
fn test_upcast_null_in_function_call() {
    // Test using upcast to pass Null to a function expecting Nullable
    take_nullable_null(Null::NULL.upcast());
}

#[wasm_bindgen_test]
fn test_upcast_undefined_in_function_call() {
    // Test using upcast to pass Undefined to a function expecting Nullable
    take_nullable_null(Undefined::UNDEFINED.upcast());
}

// Helper function that accepts Nullable via upcast
fn accepts_nullable_number(val: Nullable<Number>) -> Option<f64> {
    val.into_option().map(|n| n.value_of())
}

#[wasm_bindgen_test]
fn test_upcast_with_helper_function() {
    // Pass a Number directly via upcast
    let result = accepts_nullable_number(Number::from(99).upcast());
    assert_eq!(result, Some(99.0));

    // Pass Null via upcast
    let result = accepts_nullable_number(Null::NULL.upcast());
    assert_eq!(result, None);

    // Pass Undefined via upcast
    let result = accepts_nullable_number(Undefined::UNDEFINED.upcast());
    assert_eq!(result, None);
}
