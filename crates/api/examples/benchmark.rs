use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use structaura::{FingerprintError, Language, fingerprint_source};

const SAMPLE_COUNT: usize = 15;

struct Workload {
    name: &'static str,
    declarations: usize,
    iterations: usize,
}

const WORKLOADS: [Workload; 3] = [
    Workload { name: "small", declarations: 8, iterations: 500 },
    Workload { name: "medium", declarations: 128, iterations: 100 },
    Workload { name: "large", declarations: 1024, iterations: 10 },
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("Structaura end-to-end fingerprint benchmark");
    println!("samples={SAMPLE_COUNT}; release builds recommended");
    println!();

    for workload in WORKLOADS {
        for language in [Language::JavaScript, Language::TypeScript] {
            let source = make_source(language, workload.declarations);
            benchmark(workload.name, &source, language, workload.iterations)?;
        }
    }

    Ok(())
}

fn make_source(language: Language, declarations: usize) -> String {
    let mut source = String::new();

    if language == Language::TypeScript {
        source.push_str("type Item = { id: number; active: boolean; label: string };\n");
    }

    for index in 0..declarations {
        if language == Language::TypeScript {
            source.push_str(&format!(
                "const item_{index}: Item = {{ id: {index}, active: true, label: \"item-{index}\" }};\n"
            ));
        } else {
            source.push_str(&format!(
                "const item_{index} = {{ id: {index}, active: true, label: \"item-{index}\" }};\n"
            ));
        }
    }

    source
}

fn benchmark(
    workload: &str,
    source: &str,
    language: Language,
    iterations: usize,
) -> Result<(), FingerprintError> {
    for _ in 0..5 {
        black_box(fingerprint_source(black_box(source), language)?);
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(fingerprint_source(black_box(source), language)?);
        }
        samples.push(start.elapsed().as_nanos() / iterations as u128);
    }

    samples.sort_unstable();
    let median = samples[samples.len() / 2];
    let p95 = samples[((samples.len() * 95).div_ceil(100)).saturating_sub(1)];
    let throughput_mib = source.len() as f64 * 1_000_000_000.0 / median as f64 / (1024.0 * 1024.0);

    println!(
        "{:<7} {:<10} {:>8} bytes  median {:>10} ns/op  p95 {:>10} ns/op  {:>8.2} MiB/s",
        language.as_str(),
        workload,
        source.len(),
        median,
        p95,
        throughput_mib
    );

    Ok(())
}
