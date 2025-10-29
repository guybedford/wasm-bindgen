import { BenchmarkHarness } from './harness.js';
import * as wasm from './pkg/benchmarks_node.js';

/**
 * Main entry point for wasm-bindgen benchmarks
 * Runs all 8 realistic benchmarks and reports results
 */
async function main() {
  console.log('='.repeat(80));
  console.log('WASM-BINDGEN REALISTIC BENCHMARKS');
  console.log('Node.js ' + process.version);
  console.log('='.repeat(80));

  const harness = new BenchmarkHarness();

  try {
    // Benchmark 1: Object Graph Construction & Traversal
    await harness.runBenchmark(
      'Object Graph',
      () => wasm.bench_object_graph(),
      5,  // warmup runs
      10  // measure runs
    );

    // Benchmark 2: Collection Operations with Closures
    await harness.runBenchmark(
      'Collections',
      () => wasm.bench_collections(),
      5,
      10
    );

    // Benchmark 3: String Processing Workloads
    await harness.runBenchmark(
      'Strings',
      () => wasm.bench_strings(),
      5,
      10
    );

    // Benchmark 4: TypedArray/Buffer Operations
    await harness.runBenchmark(
      'Buffers',
      () => wasm.bench_buffers(),
      5,
      10
    );

    // Benchmark 5: Async/Promise Patterns
    await harness.runBenchmark(
      'Async',
      () => wasm.bench_async(),
      3,  // fewer warmups for async (slower)
      5   // fewer measurements for async
    );

    // Benchmark 6: Complex Type Marshalling
    await harness.runBenchmark(
      'Marshalling',
      () => wasm.bench_marshalling(),
      5,
      10
    );

    // Benchmark 7: Closure Lifecycle & Event Patterns
    await harness.runBenchmark(
      'Closures',
      () => wasm.bench_closures(),
      5,
      10
    );

    // Benchmark 8: Error Handling Paths
    await harness.runBenchmark(
      'Errors',
      () => wasm.bench_errors(),
      5,
      10
    );

    // Print summary
    harness.printSummary();

    // Optional: Export JSON for CI/automation
    if (process.argv.includes('--json')) {
      console.log('\nJSON Output:');
      console.log(JSON.stringify(harness.toJSON(), null, 2));
    }

  } catch (error) {
    console.error('Benchmark failed:', error);
    process.exit(1);
  }
}

// Run benchmarks
main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
