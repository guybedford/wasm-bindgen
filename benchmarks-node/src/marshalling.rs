use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Benchmark: Complex Type Marshalling
///
/// Tests serialization/deserialization of complex Rust types to/from JS
/// using serde-wasm-bindgen. Common in API boundaries and state management.
///
/// Tests:
/// - Nested struct serialization
/// - Enum (tagged union) handling
/// - Option<T> conversion
/// - Vec and HashMap marshalling
/// - serde-wasm-bindgen overhead
pub fn run() {
    // Test 1: Nested structs (5000 objects)
    for i in 0..5_000 {
        let user = create_user(i);
        let js_value = serde_wasm_bindgen::to_value(&user).unwrap();
        let _roundtrip: User = serde_wasm_bindgen::from_value(js_value).unwrap();
    }

    // Test 2: Enums (10,000 variants)
    for i in 0..10_000 {
        let event = create_event(i);
        let js_value = serde_wasm_bindgen::to_value(&event).unwrap();
        let _roundtrip: Event = serde_wasm_bindgen::from_value(js_value).unwrap();
    }

    // Test 3: Optional types (5000 objects with Option<T>)
    for i in 0..5_000 {
        let config = create_config(i);
        let js_value = serde_wasm_bindgen::to_value(&config).unwrap();
        let _roundtrip: Config = serde_wasm_bindgen::from_value(js_value).unwrap();
    }

    // Test 4: Collections (1000 large objects)
    for i in 0..1_000 {
        let data = create_data_collection(i, 10);
        let js_value = serde_wasm_bindgen::to_value(&data).unwrap();
        let _roundtrip: DataCollection = serde_wasm_bindgen::from_value(js_value).unwrap();
    }
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    username: String,
    email: String,
    profile: Profile,
    settings: UserSettings,
}

#[derive(Serialize, Deserialize)]
struct Profile {
    first_name: String,
    last_name: String,
    age: u32,
    avatar_url: String,
}

#[derive(Serialize, Deserialize)]
struct UserSettings {
    theme: String,
    language: String,
    notifications_enabled: bool,
    privacy_level: u32,
}

fn create_user(id: u32) -> User {
    User {
        id,
        username: format!("user{}", id),
        email: format!("user{}@example.com", id),
        profile: Profile {
            first_name: format!("First{}", id),
            last_name: format!("Last{}", id),
            age: 20 + (id % 50),
            avatar_url: format!("https://example.com/avatar/{}.png", id),
        },
        settings: UserSettings {
            theme: if id % 2 == 0 { "dark" } else { "light" }.to_string(),
            language: "en".to_string(),
            notifications_enabled: id % 3 == 0,
            privacy_level: id % 5,
        },
    }
}

#[derive(Serialize, Deserialize)]
enum Event {
    Click { x: i32, y: i32, button: u32 },
    KeyPress { key: String, ctrl: bool, shift: bool },
    Scroll { delta_x: f64, delta_y: f64 },
    Custom { event_type: String, data: String },
}

fn create_event(i: u32) -> Event {
    match i % 4 {
        0 => Event::Click { x: i as i32, y: (i * 2) as i32, button: i % 3 },
        1 => Event::KeyPress {
            key: format!("Key{}", i % 26),
            ctrl: i % 2 == 0,
            shift: i % 3 == 0,
        },
        2 => Event::Scroll {
            delta_x: (i as f64) * 0.5,
            delta_y: (i as f64) * 1.5,
        },
        _ => Event::Custom {
            event_type: format!("custom_{}", i),
            data: format!("data_{}", i),
        },
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    enabled: bool,
    max_retries: Option<u32>,
    timeout: Option<f64>,
    endpoint: Option<String>,
    headers: Option<Vec<String>>,
}

fn create_config(i: u32) -> Config {
    Config {
        name: format!("config_{}", i),
        enabled: i % 2 == 0,
        max_retries: if i % 3 == 0 { Some(i % 10) } else { None },
        timeout: if i % 4 == 0 { Some(i as f64 * 1.5) } else { None },
        endpoint: if i % 5 == 0 {
            Some(format!("https://api.example.com/v{}", i))
        } else {
            None
        },
        headers: if i % 6 == 0 {
            Some(vec![
                "Content-Type: application/json".to_string(),
                format!("X-Request-ID: {}", i),
            ])
        } else {
            None
        },
    }
}

#[derive(Serialize, Deserialize)]
struct DataCollection {
    id: u32,
    items: Vec<DataItem>,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize)]
struct DataItem {
    key: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    created_at: String,
    updated_at: String,
    version: u32,
}

fn create_data_collection(id: u32, item_count: usize) -> DataCollection {
    let items: Vec<DataItem> = (0..item_count)
        .map(|i| DataItem {
            key: format!("item_{}_{}", id, i),
            value: (id as f64) * (i as f64) * 1.234,
            tags: vec![
                format!("tag_{}", i),
                format!("category_{}", id % 10),
            ],
        })
        .collect();

    DataCollection {
        id,
        items,
        metadata: Metadata {
            created_at: "2025-10-29T12:00:00Z".to_string(),
            updated_at: "2025-10-29T12:00:00Z".to_string(),
            version: id % 100,
        },
    }
}

/// Export a function that tests marshalling across boundary
#[wasm_bindgen]
pub fn marshal_user(id: u32) -> JsValue {
    let user = create_user(id);
    serde_wasm_bindgen::to_value(&user).unwrap()
}
