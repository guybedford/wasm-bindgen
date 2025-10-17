use js_sys::{Array, JsString, Object};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/typed_array.js")]
extern "C" {
    #[derive(Clone)]
    pub type TestItem;

    #[wasm_bindgen(constructor)]
    fn new(id: u32, name: &JsString) -> TestItem;

    #[wasm_bindgen(method, getter)]
    fn id(this: &TestItem) -> u32;

    #[wasm_bindgen(method, getter)]
    fn name(this: &TestItem) -> JsString;

    #[wasm_bindgen(method)]
    fn with_prefix(this: &TestItem, prefix: &JsString) -> TestItem;
}

#[wasm_bindgen(module = "tests/wasm/typed_array.js")]
extern "C" {
    #[wasm_bindgen(js_name = "createTestItemArray")]
    fn create_test_item_array() -> Array<TestItem>;

    #[wasm_bindgen(js_name = "processTestItemArray")]
    fn process_test_item_array(arr: &Array<TestItem>) -> u32;

    #[wasm_bindgen(js_name = "checkArrayType")]
    fn check_array_type(arr: &Array<TestItem>) -> bool;
}

#[wasm_bindgen_test]
fn test_array_new_typed() {
    let arr: Array<TestItem> = Array::new_t();
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn test_array_new_with_length_typed() {
    let arr: Array<TestItem> = Array::new_with_length_t(5);
    assert_eq!(arr.length(), 5);
}

#[wasm_bindgen_test]
fn test_array_get_set() {
    let arr: Array<TestItem> = Array::new_t();
    let item = TestItem::new(1, &JsString::from("first"));

    arr.set(0, item);
    assert_eq!(arr.length(), 1);

    let retrieved: TestItem = arr.get(0);
    assert_eq!(retrieved.id(), 1);
    assert_eq!(retrieved.name(), "first");
}

#[wasm_bindgen_test]
fn test_array_at() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));
    arr.push(&TestItem::new(3, &JsString::from("c")));
    let item: TestItem = arr.at(1);
    assert_eq!(item.id(), 2);
    let last: TestItem = arr.at(-1);
    assert_eq!(last.id(), 3);
}

#[wasm_bindgen_test]
fn test_array_delete() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));

    arr.delete(0);

    assert_eq!(arr.length(), 2);
    assert!(arr.get(0).is_undefined());
}

#[wasm_bindgen_test]
fn test_array_push_pop() {
    let arr: Array<TestItem> = Array::new_t();

    let len = arr.push(&TestItem::new(1, &JsString::from("first")));
    assert_eq!(len, 1);

    let len = arr.push(&TestItem::new(2, &JsString::from("second")));
    assert_eq!(len, 2);

    let popped: TestItem = arr.pop().unwrap();
    assert_eq!(popped.id(), 2);
    assert_eq!(arr.length(), 1);

    let popped: TestItem = arr.pop().unwrap();
    assert_eq!(popped.id(), 1);
    assert_eq!(arr.length(), 0);
    assert!(arr.pop().is_none());
}

#[wasm_bindgen_test]
fn test_array_shift() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("first")));
    arr.push(&TestItem::new(2, &JsString::from("second")));

    let shifted: TestItem = arr.shift();
    assert_eq!(shifted.id(), 1);
    assert_eq!(arr.length(), 1);

    let remaining: TestItem = arr.get(0);
    assert_eq!(remaining.id(), 2);
    arr.shift();
    assert!(arr.shift_checked().is_none());
}

#[wasm_bindgen_test]
fn test_array_concat() {
    let arr1: Array<TestItem> = Array::new_t();
    arr1.push(&TestItem::new(1, &JsString::from("a")));

    let arr2: Array<TestItem> = Array::new_t();
    arr2.push(&TestItem::new(2, &JsString::from("b")));

    let combined = arr1.concat(&arr2);
    assert_eq!(combined.length(), 2);

    let first: TestItem = combined.get(0);
    let second: TestItem = combined.get(1);
    assert_eq!(first.id(), 1);
    assert_eq!(second.id(), 2);
}

#[wasm_bindgen_test]
fn test_array_reverse() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));
    arr.push(&TestItem::new(3, &JsString::from("c")));

    let reversed = arr.reverse();

    let first: TestItem = reversed.get(0);
    let last: TestItem = reversed.get(2);
    assert_eq!(first.id(), 3);
    assert_eq!(last.id(), 1);
}

#[wasm_bindgen_test]
fn test_array_copy_within() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));
    arr.push(&TestItem::new(3, &JsString::from("c")));
    arr.push(&TestItem::new(4, &JsString::from("d")));
    let result = arr.copy_within(2, 0, 2);

    let at2: TestItem = result.get(2);
    let at3: TestItem = result.get(3);
    assert_eq!(at2.id(), 1);
    assert_eq!(at3.id(), 2);
}

#[wasm_bindgen_test]
fn test_array_iter() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));
    arr.push(&TestItem::new(3, &JsString::from("c")));

    let ids: Vec<u32> = arr
        .iter()
        .map(|item| {
            let t: TestItem = item;
            t.id()
        })
        .collect();

    assert_eq!(ids, vec![1, 2, 3]);
}

#[wasm_bindgen_test]
fn test_array_into_iter() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(10, &JsString::from("x")));
    arr.push(&TestItem::new(20, &JsString::from("y")));

    let mut sum = 0;
    for item in arr {
        let t: TestItem = item;
        sum += t.id();
    }
    assert_eq!(sum, 30);
}

#[wasm_bindgen_test]
fn test_array_to_vec() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));

    let vec: Vec<TestItem> = arr.to_vec();
    assert_eq!(vec.len(), 2);

    let first = vec[0].clone();
    let second = vec[1].clone();
    assert_eq!(first.id(), 1);
    assert_eq!(second.id(), 2);
}

#[wasm_bindgen_test]
fn test_array_from_iter() {
    let items = vec![
        TestItem::new(1, &JsString::from("a")),
        TestItem::new(2, &JsString::from("b")),
        TestItem::new(3, &JsString::from("c")),
    ];

    let arr: Array<TestItem> = items.iter().map(|i| i).collect();

    assert_eq!(arr.length(), 3);
    let first: TestItem = arr.get(0);
    assert_eq!(first.id(), 1);
}

#[wasm_bindgen_test]
fn test_array_extend() {
    let mut arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));

    let more = vec![
        TestItem::new(2, &JsString::from("b")),
        TestItem::new(3, &JsString::from("c")),
    ];
    arr.extend(more);

    assert_eq!(arr.length(), 3);
}

#[wasm_bindgen_test]
fn test_array_find() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("apple")));
    arr.push(&TestItem::new(2, &JsString::from("banana")));
    arr.push(&TestItem::new(3, &JsString::from("cherry")));

    let found = arr.find(&mut |val, _, _| {
        let item: TestItem = val.into();
        item.id() == 2
    });

    let item: TestItem = found;
    assert_eq!(item.name(), "banana");
}

#[wasm_bindgen_test]
fn test_array_find_index() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));
    arr.push(&TestItem::new(2, &JsString::from("b")));
    arr.push(&TestItem::new(3, &JsString::from("c")));

    let idx = arr.find_index(&mut |val, _, _| {
        let item: TestItem = val.into();
        item.id() == 2
    });
    assert_eq!(idx, 1);

    let not_found = arr.find_index(&mut |val, _, _| {
        let item: TestItem = val.into();
        item.id() == 99
    });
    assert_eq!(not_found, -1);
}

#[wasm_bindgen_test]
fn test_array_every() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(2, &JsString::from("a")));
    arr.push(&TestItem::new(4, &JsString::from("b")));
    arr.push(&TestItem::new(6, &JsString::from("c")));

    let all_even = arr.every(&mut |val, _, _| {
        let item: TestItem = val.into();
        item.id() % 2 == 0
    });
    assert!(all_even);

    arr.push(&TestItem::new(7, &JsString::from("d")));

    let still_all_even = arr.every(&mut |val, _, _| {
        let item: TestItem = val.into();
        item.id() % 2 == 0
    });
    assert!(!still_all_even);
}

#[wasm_bindgen_test]
fn test_array_from_js() {
    let arr = create_test_item_array();

    assert_eq!(arr.length(), 3);

    let first: TestItem = arr.get(0);
    assert_eq!(first.id(), 1);
}

#[wasm_bindgen_test]
fn test_array_to_js() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(10, &JsString::from("x")));
    arr.push(&TestItem::new(20, &JsString::from("y")));

    let sum = process_test_item_array(&arr);
    assert_eq!(sum, 30);
}

#[wasm_bindgen_test]
fn test_array_type_preserved() {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(1, &JsString::from("a")));

    assert!(check_array_type(&arr));
}

#[wasm_bindgen]
pub fn rust_create_test_item_array() -> Array<TestItem> {
    let arr: Array<TestItem> = Array::new_t();
    arr.push(&TestItem::new(100, &JsString::from("rust1")));
    arr.push(&TestItem::new(200, &JsString::from("rust2")));
    arr
}

#[wasm_bindgen_test]
fn test_rust_export_typed_array() {
    let arr = rust_create_test_item_array();
    assert_eq!(arr.length(), 2);

    let first: TestItem = arr.get(0);
    assert_eq!(first.id(), 100);
    assert_eq!(first.name(), "rust1");
}
