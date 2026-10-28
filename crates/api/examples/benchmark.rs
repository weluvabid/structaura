use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use structaura::{FingerprintError, Language, fingerprint_source};
use structaura_core::{CanonicalNode, encode_node_for_benchmark, hash_node};
use tree_sitter::{Node, Tree};

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

struct EncodingPlan<'a> {
    node: &'a CanonicalNode,
    child_hashes: Vec<[u8; 32]>,
    children: Vec<EncodingPlan<'a>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Structaura end-to-end fingerprint benchmark");
    println!("samples={SAMPLE_COUNT}; release builds recommended");
    println!();

    for workload in WORKLOADS {
        for language in [Language::JavaScript, Language::TypeScript] {
            let source = make_declarations(language, workload.declarations);
            benchmark_end_to_end(workload.name, &source, language, workload.iterations)?;
        }
    }

    println!("\nStage investigation (medium workloads)");
    println!("Stage timings are diagnostic and are not additive.");
    println!();

    for language in [Language::JavaScript, Language::TypeScript] {
        for shape in ["declarations", "nested", "functions"] {
            let source = make_shaped_source(language, shape, 128);
            benchmark_stages(language, shape, &source)?;
        }
    }

    Ok(())
}

fn make_declarations(language: Language, declarations: usize) -> String {
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

fn make_shaped_source(language: Language, shape: &str, count: usize) -> String {
    match shape {
        "declarations" => make_declarations(language, count),
        "nested" => {
            let mut source = String::new();
            for index in 0..count {
                if language == Language::TypeScript {
                    source.push_str(&format!(
                        "const value_{index}: number = ((({index} + 1) * 2) - 3) / 4;\n"
                    ));
                } else {
                    source.push_str(&format!(
                        "const value_{index} = ((({index} + 1) * 2) - 3) / 4;\n"
                    ));
                }
            }
            source
        }
        "functions" => {
            let mut source = String::new();
            for index in 0..count {
                if language == Language::TypeScript {
                    source.push_str(&format!(
                        "function transform_{index}(value: number): number {{ const doubled = value * 2; return doubled + {index}; }}\n"
                    ));
                } else {
                    source.push_str(&format!(
                        "function transform_{index}(value) {{ const doubled = value * 2; return doubled + {index}; }}\n"
                    ));
                }
            }
            source
        }
        _ => unreachable!("benchmark shape is defined in the harness"),
    }
}

fn benchmark_end_to_end(
    workload: &str,
    source: &str,
    language: Language,
    iterations: usize,
) -> Result<(), FingerprintError> {
    let samples = measure(iterations, || {
        black_box(fingerprint_source(black_box(source), language).unwrap());
    });
    print_measurement(language.as_str(), workload, source.len(), &samples);
    Ok(())
}

fn benchmark_stages(language: Language, shape: &str, source: &str) -> Result<(), Box<dyn Error>> {
    let tree = parse_source(source, language)?;
    let canonical = canonicalize_tree(tree.root_node(), source, language)?;
    let plan = build_encoding_plan(&canonical);
    let iterations = 100;

    let parse_samples = measure(iterations, || {
        black_box(parse_source(source, language).unwrap());
    });
    let canonical_samples = measure(iterations, || {
        black_box(canonicalize_tree(tree.root_node(), source, language).unwrap());
    });
    let encoding_samples = measure(iterations, || {
        let mut encoded = Vec::new();
        encode_plan(&plan, &mut encoded);
        black_box(encoded);
    });
    let merkle_samples = measure(iterations, || {
        black_box(hash_node(&canonical));
    });
    let end_to_end_samples = measure(iterations, || {
        black_box(fingerprint_source(source, language).unwrap());
    });

    println!(
        "{} {:<12} {:>7} bytes | parse {:>9} ns | canonicalize {:>9} ns | encode {:>9} ns | merkle {:>9} ns | end-to-end {:>9} ns",
        language.as_str(),
        shape,
        source.len(),
        median(&parse_samples),
        median(&canonical_samples),
        median(&encoding_samples),
        median(&merkle_samples),
        median(&end_to_end_samples),
    );

    Ok(())
}

fn parse_source(source: &str, language: Language) -> Result<Tree, Box<dyn Error>> {
    match language {
        Language::JavaScript => Ok(structaura_javascript::parse(source)?),
        Language::TypeScript => Ok(structaura_typescript::parse(source)?),
    }
}

fn canonicalize_tree(
    root: Node<'_>,
    source: &str,
    language: Language,
) -> Result<CanonicalNode, Box<dyn Error>> {
    match language {
        Language::JavaScript => {
            Ok(structaura_javascript::canonicalize_for_benchmark(root, source)?)
        }
        Language::TypeScript => {
            Ok(structaura_typescript::canonicalize_for_benchmark(root, source)?)
        }
    }
}

fn build_encoding_plan(node: &CanonicalNode) -> EncodingPlan<'_> {
    EncodingPlan {
        node,
        child_hashes: node.children().iter().map(hash_node).collect(),
        children: node.children().iter().map(build_encoding_plan).collect(),
    }
}

fn encode_plan(plan: &EncodingPlan<'_>, output: &mut Vec<Vec<u8>>) {
    output.push(encode_node_for_benchmark(plan.node, &plan.child_hashes));
    for child in &plan.children {
        encode_plan(child, output);
    }
}

fn measure<F>(iterations: usize, mut operation: F) -> Vec<u128>
where
    F: FnMut(),
{
    for _ in 0..5 {
        operation();
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        samples.push(start.elapsed().as_nanos() / iterations as u128);
    }

    samples.sort_unstable();
    samples
}

fn median(samples: &[u128]) -> u128 {
    samples[samples.len() / 2]
}

fn print_measurement(language: &str, workload: &str, source_size: usize, samples: &[u128]) {
    let median = median(samples);
    let p95 = samples[((samples.len() * 95).div_ceil(100)).saturating_sub(1)];
    let throughput_mib = source_size as f64 * 1_000_000_000.0 / median as f64 / (1024.0 * 1024.0);

    println!(
        "{:<7} {:<10} {:>8} bytes  median {:>10} ns/op  p95 {:>10} ns/op  {:>8.2} MiB/s",
        language, workload, source_size, median, p95, throughput_mib
    );
}
