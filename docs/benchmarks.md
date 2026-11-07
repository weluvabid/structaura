# Benchmark results

StructAura's benchmark measures the complete source-to-fingerprint path for JavaScript and
TypeScript, then provides stage-level timings to make the processing pipeline observable.

## Results

The following results were captured using an optimized Rust release build. Each workload was
measured over 15 samples; each reported operation latency is the average within one sample.
Median and p95 are calculated across those samples after five warm-up operations.

| Language | Workload | Source size | Median | p95 | Throughput |
|---|---|---:|---:|---:|---:|
| JavaScript | Small | 456 B | 88.972 µs | 140.802 µs | 4.89 MiB/s |
| TypeScript | Small | 564 B | 115.520 µs | 116.562 µs | 4.66 MiB/s |
| JavaScript | Medium | 7,734 B | 1.445 ms | 1.459 ms | 5.10 MiB/s |
| TypeScript | Medium | 8,562 B | 1.656 ms | 1.679 ms | 4.93 MiB/s |
| JavaScript | Large | 64,254 B | 11.568 ms | 12.031 ms | 5.30 MiB/s |
| TypeScript | Large | 70,458 B | 13.203 ms | 14.005 ms | 5.09 MiB/s |

Across these representative inputs, the end-to-end API fingerprints source files from hundreds of
bytes to roughly 70 KB. Small inputs complete in under 116 µs median latency, while the largest
measured inputs complete in approximately 12–14 ms at p95.

## Pipeline visibility

The benchmark also measures the stages independently on medium workloads. These timings are
diagnostic and **must not be summed**: stage instrumentation changes execution characteristics,
and the stage measurements are not an additive decomposition of the end-to-end result.

| Language | Workload | Parse | Canonicalize | Encode | Merkle | End-to-end |
|---|---|---:|---:|---:|---:|---:|
| JavaScript | Declarations | 484.030 µs | 478.516 µs | 221.022 µs | 491.784 µs | 1.456 ms |
| JavaScript | Nested | 527.221 µs | 680.887 µs | 254.663 µs | 574.778 µs | 1.800 ms |
| JavaScript | Functions | 556.133 µs | 512.121 µs | 318.101 µs | 684.980 µs | 1.752 ms |
| TypeScript | Declarations | 551.592 µs | 537.414 µs | 256.804 µs | 568.254 µs | 1.659 ms |
| TypeScript | Nested | 600.304 µs | 759.728 µs | 312.449 µs | 667.797 µs | 2.036 ms |
| TypeScript | Functions | 720.168 µs | 685.579 µs | 414.672 µs | 885.332 µs | 2.258 ms |

This stage view makes the architecture measurable: parsing, canonicalization, deterministic
encoding, and Merkle hashing each have an observable contribution to the fingerprinting pipeline.

## Method

Run the benchmark from the repository root:

```sh
./scripts/bench.sh
```

The harness builds and runs the native API benchmark in release mode. It covers generated
JavaScript and TypeScript inputs at small, medium, and large sizes, and includes declaration,
nested-structure, and function-heavy medium workloads for stage investigation.

To capture a fresh run:

```sh
./scripts/bench.sh 2>&1 | tee benchmark-results.txt
```

The recorded values above are one benchmark run, not a cross-machine comparison. They describe
the observed execution for this run and should be interpreted alongside the workload sizes,
sample count, warm-up count, and release-build configuration. Hardware details are intentionally
not published.

For repeatable comparisons, use the same source fixtures, Rust toolchain, build profile, and
measurement procedure across runs. Compare like-for-like results and retain the raw output with
the release record.
