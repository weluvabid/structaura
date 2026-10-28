# Structaura

Deterministic AST fingerprinting for JavaScript and TypeScript. Ignore formatting changes, detect structural changes.


## Benchmarks

Run the end-to-end JavaScript and TypeScript fingerprinting benchmark in release mode:

```sh
./scripts/bench.sh
```

The harness measures the public API across small, medium, and large generated source files.
It reports source size, median latency, p95 sample latency, and input throughput. Each reported
latency is the per-operation average within a sample; the median and p95 are calculated across
15 samples after five warm-up operations.

For comparable results, record `rustc -Vv`, the operating system, and CPU model, and run on an
otherwise idle machine. Compare results from the same machine and toolchain. These numbers
measure the full parse-to-fingerprint path; they are not isolated parser or hashing timings.
