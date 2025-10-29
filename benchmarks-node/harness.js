/**
 * Lightweight benchmark harness for wasm-bindgen benchmarks
 * Inspired by JetStream 3 methodology
 */

export class BenchmarkHarness {
  constructor() {
    this.results = [];
  }

  /**
   * Run a benchmark with warmup and measurement phases
   * @param {string} name - Benchmark name
   * @param {Function} fn - Benchmark function (can be async)
   * @param {number} warmupRuns - Number of warmup iterations
   * @param {number} measureRuns - Number of measured iterations
   */
  async runBenchmark(name, fn, warmupRuns = 5, measureRuns = 10) {
    console.log(`\n[${name}]`);
    console.log(`  Warming up (${warmupRuns} runs)...`);

    // Warmup phase - stabilize JIT
    for (let i = 0; i < warmupRuns; i++) {
      await fn();
    }

    console.log(`  Measuring (${measureRuns} runs)...`);

    // Measurement phase
    const times = [];
    for (let i = 0; i < measureRuns; i++) {
      const start = performance.now();
      await fn();
      const end = performance.now();
      times.push(end - start);
    }

    // Calculate statistics
    const stats = this.calculateStats(name, times);
    this.results.push(stats);

    // Print results
    console.log(`  Mean: ${stats.mean.toFixed(3)}ms`);
    console.log(`  Median: ${stats.median.toFixed(3)}ms`);
    console.log(`  Min: ${stats.min.toFixed(3)}ms`);
    console.log(`  Max: ${stats.max.toFixed(3)}ms`);
    console.log(`  StdDev: ${stats.stddev.toFixed(3)}ms`);
    console.log(`  Ops/sec: ${stats.opsPerSec.toFixed(2)}`);

    return stats;
  }

  /**
   * Calculate statistical measures from timing samples
   */
  calculateStats(name, times) {
    const sorted = times.slice().sort((a, b) => a - b);
    const n = times.length;

    const mean = times.reduce((a, b) => a + b, 0) / n;
    const median = n % 2 === 0
      ? (sorted[n / 2 - 1] + sorted[n / 2]) / 2
      : sorted[Math.floor(n / 2)];
    const min = sorted[0];
    const max = sorted[n - 1];

    // Calculate standard deviation
    const variance = times.reduce((sum, t) => sum + Math.pow(t - mean, 2), 0) / n;
    const stddev = Math.sqrt(variance);

    // Operations per second (1000ms / mean time per operation)
    const opsPerSec = 1000 / mean;

    return {
      name,
      mean,
      median,
      min,
      max,
      stddev,
      opsPerSec,
      samples: n
    };
  }

  /**
   * Print summary table of all results
   */
  printSummary() {
    console.log('\n' + '='.repeat(80));
    console.log('BENCHMARK SUMMARY');
    console.log('='.repeat(80));
    console.log();

    // Find max name length for formatting
    const maxNameLen = Math.max(...this.results.map(r => r.name.length));

    // Header
    console.log(
      this.pad('Benchmark', maxNameLen) + '  ' +
      this.pad('Mean (ms)', 12, true) + '  ' +
      this.pad('Median (ms)', 12, true) + '  ' +
      this.pad('Ops/sec', 12, true)
    );
    console.log('-'.repeat(80));

    // Results
    for (const result of this.results) {
      console.log(
        this.pad(result.name, maxNameLen) + '  ' +
        this.pad(result.mean.toFixed(3), 12, true) + '  ' +
        this.pad(result.median.toFixed(3), 12, true) + '  ' +
        this.pad(result.opsPerSec.toFixed(2), 12, true)
      );
    }

    console.log('='.repeat(80));
  }

  /**
   * Helper for padding strings
   */
  pad(str, length, right = false) {
    str = String(str);
    if (right) {
      return str.padStart(length);
    }
    return str.padEnd(length);
  }

  /**
   * Export results as JSON
   */
  toJSON() {
    return {
      benchmarks: this.results,
      timestamp: new Date().toISOString(),
      runtime: 'Node.js ' + process.version
    };
  }
}
