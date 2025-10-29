# Wasm-Bindgen Realistic Benchmarks

A comprehensive benchmark suite for wasm-bindgen running in Node.js. These benchmarks measure real-world usage patterns at scale, inspired by JetStream 3's methodology.

## Philosophy

Unlike micro-benchmarks that test individual function call overhead, these benchmarks focus on **realistic workloads** that represent actual wasm-bindgen usage:

- **Composite operations**: Multiple operations combined like real applications
- **Realistic data volumes**: 1000s-10000s of operations per benchmark
- **Real patterns**: Extracted from actual wasm-bindgen examples and applications
- **Pure JS standard library**: No web or Node-specific APIs for portability

## Benchmarks

### 1. Object Graph Construction & Traversal
**What it tests**: Building and walking complex nested JS objects from Rust

**Workload**: Creates a tree with ~88,000 nodes (8 levels deep, 3 children per node), then traverses to count nodes and sum values.

**Real-world scenario**: Component trees, virtual DOM, configuration objects, data models

**wasm-bindgen features**: `js_sys::Object`, `Reflect::set/get`, structural typing

---

### 2. Collection Operations with Closures
**What it tests**: Processing arrays with Rust closures passed to JS Array methods

**Workload**: 10,000-item arrays processed with map, filter, reduce operations plus complex pipelines

**Real-world scenario**: Data processing pipelines, state transformations, reactive updates

**wasm-bindgen features**: `Closure::wrap`, `js_sys::Array`, type conversions

---

### 3. String Processing Workloads
**What it tests**: String marshalling at various sizes and encodings

**Workload**:
- 1,000 template renderings
- 500 URL manipulations
- 5,000 structured log messages
- 1,000 Unicode messages (emoji + Japanese)
- Large string concatenation

**Real-world scenario**: Template engines, logging, text processing, internationalization

**wasm-bindgen features**: UTF-8 ↔ UTF-16 conversion, string allocation/deallocation

---

### 4. TypedArray/Buffer Operations
**What it tests**: Zero-copy data transfer and typed array manipulation

**Workload**:
- 1920×1080 RGBA image buffer processing (8.3M bytes)
- 48kHz 10-second stereo audio (960K samples)
- 1MB binary data transfer

**Real-world scenario**: Canvas operations, image processing, audio synthesis, media manipulation

**wasm-bindgen features**: `Uint8ClampedArray`, `Float32Array`, buffer views

---

### 5. Async/Promise Patterns
**What it tests**: Future→Promise overhead and async runtime cost

**Workload**:
- 1,000 sequential promise operations
- 100 concurrent promises
- 50 levels of nested async calls

**Real-world scenario**: API calls, async data processing, fetch operations

**wasm-bindgen features**: `wasm-bindgen-futures`, `JsFuture`, `Promise` integration

---

### 6. Complex Type Marshalling
**What it tests**: serde-wasm-bindgen and JsValue conversion overhead

**Workload**:
- 5,000 nested struct serializations
- 10,000 enum (tagged union) conversions
- 5,000 Option<T> handling
- 1,000 large collections with nested data

**Real-world scenario**: API boundaries, state serialization, configuration passing

**wasm-bindgen features**: `serde-wasm-bindgen`, `#[derive(Serialize, Deserialize)]`

---

### 7. Closure Lifecycle & Event Patterns
**What it tests**: Closure creation, invocation, and memory management

**Workload**:
- 10,000 short-lived closures
- 1,000 event handlers with 10 invocations each
- 5,000 stateful closures
- 1,000 closure compositions

**Real-world scenario**: Event-driven architectures, callbacks, reactive systems

**wasm-bindgen features**: `Closure::new()`, `Closure::forget()`, captured state

---

### 8. Error Handling Paths
**What it tests**: Result<T,E> and exception propagation overhead

**Workload**:
- 5,000 operations with 10% error rate
- 1,000 error propagations through call stack
- 10,000 try/catch operations
- 1,000 complex error type conversions

**Real-world scenario**: Robust error handling, validation, fault tolerance

**wasm-bindgen features**: `Result<T, JsValue>`, exception throwing/catching

---

## Building and Running

### Prerequisites

```bash
# Rust with wasm target
rustup target add wasm32-unknown-unknown

# wasm-opt (optional, for optimization)
# macOS: brew install binaryen
# Ubuntu: apt-get install binaryen
# Or download from: https://github.com/WebAssembly/binaryen/releases

# Note: wasm-bindgen-cli is built from the workspace, no need to install separately
```

### Build

```bash
cd benchmarks-node
npm run build
```

This runs:
1. `cargo build --release --target wasm32-unknown-unknown` - Compile Rust to WASM
2. `cargo run --manifest-path ../crates/cli/Cargo.toml` - Generate JS bindings using wasm-bindgen from workspace
3. `wasm-opt -O3` (optional) - Optimize WASM binary if available

### Run Benchmarks

```bash
npm run bench
```

Optional: Export results as JSON for CI integration:

```bash
npm run bench -- --json
```

## Results Interpretation

The harness reports:
- **Mean**: Average time per benchmark iteration
- **Median**: Middle value (less affected by outliers)
- **Min/Max**: Best and worst times
- **StdDev**: Consistency of results (lower is more stable)
- **Ops/sec**: Operations per second (higher is better)

Each benchmark includes:
- **Warmup phase** (5-10 runs): Stabilizes JIT compilation
- **Measurement phase** (5-10 runs): Actual timing

## Design Decisions

### Why Single Rust Build?
- Simpler build process
- All benchmarks share the same wasm-bindgen version and settings
- Easier to maintain consistency
- Each benchmark is a separate exported function

### Why Manual Build Pipeline?
- Full control over optimization levels
- Explicit wasm-opt configuration
- Easier to debug and customize
- No hidden wasm-pack magic

### Why Bundler Target?
- Optimal for Node.js ES modules
- Clean import/export semantics
- Modern JavaScript output
- No web API assumptions

### Why These Workload Sizes?
Calibrated for:
- **1-2 seconds per benchmark**: Long enough to measure accurately, short enough to iterate quickly
- **JetStream 3 scale**: Similar data volumes to industry-standard benchmarks
- **Real-world representativeness**: Sizes that appear in actual applications

## Extending

To add a new benchmark:

1. Create `src/new_benchmark.rs` with `pub fn run()` implementation
2. Add module declaration in `src/lib.rs`: `mod new_benchmark;`
3. Export function in `src/lib.rs`:
   ```rust
   #[wasm_bindgen]
   pub fn bench_new_feature() {
       new_benchmark::run();
   }
   ```
4. Add to `index.js`:
   ```javascript
   await harness.runBenchmark(
     'New Feature',
     () => wasm.bench_new_feature(),
     5, 10
   );
   ```

## CI Integration

The `--json` flag outputs structured data suitable for tracking over time:

```json
{
  "benchmarks": [
    {
      "name": "Object Graph",
      "mean": 123.456,
      "median": 122.0,
      "opsPerSec": 8.1
    }
  ],
  "timestamp": "2025-10-29T12:34:56.789Z",
  "runtime": "Node.js v20.10.0"
}
```

Use this to:
- Track performance regressions
- Compare across wasm-bindgen versions
- A/B test optimization strategies
- Generate performance dashboards

## License

Same as wasm-bindgen (MIT/Apache-2.0)
