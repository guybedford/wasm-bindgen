use wasm_bindgen::prelude::*;

/// Benchmark: String Processing Workloads
///
/// Simulates common string operations: template rendering, URL manipulation,
/// log formatting with mixed ASCII/Unicode content.
///
/// Tests:
/// - String allocation and deallocation
/// - UTF-8 ↔ UTF-16 conversion overhead
/// - String concatenation patterns
/// - Various string sizes (small, medium, large)
/// - Unicode handling
pub fn run() {
    // Test 1: Template rendering (1000 iterations)
    let mut total_len = 0;
    for i in 0..1_000 {
        let rendered = render_template("User", i, "logged in successfully");
        total_len += rendered.len();
    }
    assert!(total_len > 0);

    // Test 2: URL manipulation (500 iterations)
    let mut url_count = 0;
    for i in 0..500 {
        let url = build_url("https://api.example.com", "users", i);
        if url.contains("api.example.com") {
            url_count += 1;
        }
    }
    assert_eq!(url_count, 500);

    // Test 3: Log formatting (5000 structured messages)
    let mut log_total = 0;
    for i in 0..5_000 {
        let log = format_log_message("INFO", "benchmark", "Processing item", i);
        log_total += log.len();
    }
    assert!(log_total > 0);

    // Test 4: Unicode handling (1000 iterations with emoji and Japanese)
    let mut unicode_len = 0;
    for i in 0..1_000 {
        let msg = format_unicode_message(i);
        unicode_len += msg.len();
    }
    assert!(unicode_len > 0);

    // Test 5: Large string concatenation
    let large = build_large_string(100);
    assert!(large.len() > 10_000);
}

/// Simulate template string interpolation
#[inline(never)]
fn render_template(name: &str, id: u32, action: &str) -> String {
    format!("[{}] User '{}' (ID: {}) {}",
        get_timestamp(), name, id, action)
}

/// Build a URL with path and query params
#[inline(never)]
fn build_url(base: &str, path: &str, id: u32) -> String {
    format!("{}/{}?id={}&format=json&timestamp={}",
        base, path, id, get_timestamp())
}

/// Format a structured log message
#[inline(never)]
fn format_log_message(level: &str, module: &str, message: &str, value: u32) -> String {
    format!("[{}] [{}] {} - value={}, hex=0x{:x}, bin={:b}",
        level, module, message, value, value, value)
}

/// Format message with Unicode (emoji + Japanese)
#[inline(never)]
fn format_unicode_message(count: u32) -> String {
    format!("🚀 ベンチマーク #{}: テスト実行中... ✅ Complete!", count)
}

/// Build a large string by concatenation
#[inline(never)]
fn build_large_string(iterations: u32) -> String {
    let mut result = String::with_capacity(iterations as usize * 200);

    for i in 0..iterations {
        result.push_str(&format!(
            "Line {}: This is a longer line of text that simulates a real-world string \
            concatenation pattern. It includes numbers like {} and formatting.\n",
            i, i * 100
        ));
    }

    result
}

/// Simulate getting a timestamp string
#[inline(never)]
fn get_timestamp() -> String {
    // In real code this would call Date.now() or similar
    // For benchmarking, we just generate a consistent format
    "2025-10-29T12:34:56.789Z".to_string()
}

/// Test round-tripping strings through JS boundary
#[wasm_bindgen]
pub fn string_roundtrip(input: String) -> String {
    // This tests the overhead of passing strings across the boundary
    input.to_uppercase()
}
