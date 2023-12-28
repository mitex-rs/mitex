//! Simple benchmarks for the parser.
//! It has simple manual constructed inputs so doesn't really show the
//! performance of the parser. Insteadly, it is for tracking performance changes
//! for developers.

use divan::{AllocProfiler, Bencher};
use mitex_parser::{parse, parse_without_macro, CommandSpec};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

// The default spec used for testing
fn default_spec() -> CommandSpec {
    use mitex_parser::command_preludes::*;
    let mut builder = SpecBuilder::default();
    builder.add_command("alpha", TEX_SYMBOL);
    builder.build()
}

fn heavy_spec() -> CommandSpec {
    use mitex_parser::command_preludes::*;
    let mut builder = SpecBuilder::default();
    builder.add_command("alpha", TEX_SYMBOL);
    for i in 0..1000 {
        builder.add_command(&format!("alpha{i}"), TEX_SYMBOL);
    }
    builder.build()
}

static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(default_spec);
static HEAVY_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(heavy_spec);

static ALPHA_X_20000: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\alpha x".repeat(20000));

static ALPHA_X_40000: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\alpha x".repeat(40000));

fn bench(bencher: Bencher, input: &str, spec: CommandSpec) {
    bencher.bench(|| parse(input, spec.clone()));
}

fn bench_no_macro(bencher: Bencher, input: &str, spec: CommandSpec) {
    bencher.bench(|| parse_without_macro(input, spec.clone()));
}

#[divan::bench]
fn alpha_x_20000(bencher: Bencher) {
    bench(bencher, &ALPHA_X_20000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000(bencher: Bencher) {
    bench(bencher, &ALPHA_X_40000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000_no_macro(bencher: Bencher) {
    bench_no_macro(bencher, &ALPHA_X_40000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_20000_heavy(bencher: Bencher) {
    bench(bencher, &ALPHA_X_20000, HEAVY_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000_heavy(bencher: Bencher) {
    bench(bencher, &ALPHA_X_40000, HEAVY_SPEC.clone());
}

static SLICE_WORD: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\frac ab ".repeat(20000));

#[divan::bench]
fn slice_word(bencher: Bencher) {
    bench(bencher, &SLICE_WORD, DEFAULT_SPEC.clone());
}

static SQRT_PATTERN: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\sqrt[1]{2} \\sqrt{2} \\sqrt{2} \\sqrt{2}".repeat(2500));

#[divan::bench]
fn sqrt_pattern(bencher: Bencher) {
    bench(bencher, &SQRT_PATTERN, DEFAULT_SPEC.clone());
}

static PLAIN_TEXT: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "hello world ".repeat(5000));

#[divan::bench]
fn plain_text(bencher: Bencher) {
    bench(bencher, &PLAIN_TEXT, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn plain_text_no_macro(bencher: Bencher) {
    bench_no_macro(bencher, &PLAIN_TEXT, DEFAULT_SPEC.clone());
}

static STARRED_COMMAND: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\operatorname*{a}".repeat(5000));

#[divan::bench]
fn starred_command(bencher: Bencher) {
    bench(bencher, &STARRED_COMMAND, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn starred_command_no_macro(bencher: Bencher) {
    bench_no_macro(bencher, &STARRED_COMMAND, DEFAULT_SPEC.clone());
}

/*
last^1
simple                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ alpha_x_20000          1.957 ms      │ 3.166 ms      │ 2.169 ms      │ 2.243 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           12          │ 12            │ 12            │ 12            │         │
│                           960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                         dealloc:      │               │               │               │         │
│                           5           │ 5             │ 5             │ 5             │         │
│                           1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                         grow:         │               │               │               │         │
│                           18          │ 18            │ 18            │ 18            │         │
│                           1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ alpha_x_20000_heavy    2.026 ms      │ 3.064 ms      │ 2.282 ms      │ 2.362 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           12          │ 12            │ 12            │ 12            │         │
│                           960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                         dealloc:      │               │               │               │         │
│                           5           │ 5             │ 5             │ 5             │         │
│                           1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                         grow:         │               │               │               │         │
│                           18          │ 18            │ 18            │ 18            │         │
│                           1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ alpha_x_40000          4.328 ms      │ 6.038 ms      │ 4.66 ms       │ 4.766 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           12          │ 12            │ 12            │ 12            │         │
│                           1.92 MB     │ 1.92 MB       │ 1.92 MB       │ 1.92 MB       │         │
│                         dealloc:      │               │               │               │         │
│                           5           │ 5             │ 5             │ 5             │         │
│                           3.152 MB    │ 3.152 MB      │ 3.152 MB      │ 3.152 MB      │         │
│                         grow:         │               │               │               │         │
│                           19          │ 19            │ 19            │ 19            │         │
│                           3.151 MB    │ 3.151 MB      │ 3.151 MB      │ 3.151 MB      │         │
├─ alpha_x_40000_heavy    4.485 ms      │ 6.775 ms      │ 4.962 ms      │ 5.033 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           12          │ 12            │ 12            │ 12            │         │
│                           1.92 MB     │ 1.92 MB       │ 1.92 MB       │ 1.92 MB       │         │
│                         dealloc:      │               │               │               │         │
│                           5           │ 5             │ 5             │ 5             │         │
│                           3.152 MB    │ 3.152 MB      │ 3.152 MB      │ 3.152 MB      │         │
│                         grow:         │               │               │               │         │
│                           19          │ 19            │ 19            │ 19            │         │
│                           3.151 MB    │ 3.151 MB      │ 3.151 MB      │ 3.151 MB      │         │
├─ alpha_x_40000_macro    4.616 ms      │ 7.79 ms       │ 5.089 ms      │ 5.19 ms       │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           57          │ 57            │ 57            │ 57            │         │
│                           1.924 MB    │ 1.924 MB      │ 1.924 MB      │ 1.924 MB      │         │
│                         dealloc:      │               │               │               │         │
│                           50          │ 50            │ 50            │ 50            │         │
│                           3.156 MB    │ 3.156 MB      │ 3.156 MB      │ 3.156 MB      │         │
│                         grow:         │               │               │               │         │
│                           22          │ 22            │ 22            │ 22            │         │
│                           3.152 MB    │ 3.152 MB      │ 3.152 MB      │ 3.152 MB      │         │
├─ slice_word             2.057 ms      │ 3.94 ms       │ 2.314 ms      │ 2.411 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           12          │ 12            │ 12            │ 12            │         │
│                           960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                         dealloc:      │               │               │               │         │
│                           5           │ 5             │ 5             │ 5             │         │
│                           1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                         grow:         │               │               │               │         │
│                           18          │ 18            │ 18            │ 18            │         │
│                           1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ sqrt_pattern           1.778 ms      │ 4.774 ms      │ 2.04 ms       │ 2.225 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           7523        │ 7523          │ 7523          │ 7523          │         │
│                           1.021 MB    │ 1.021 MB      │ 1.021 MB      │ 1.021 MB      │         │
│                         dealloc:      │               │               │               │         │
│                           8           │ 8             │ 8             │ 8             │         │
│                           793 KB      │ 793 KB        │ 793 KB        │ 793 KB        │         │
│                         grow:         │               │               │               │         │
│                           17          │ 17            │ 17            │ 17            │         │
│                           792 KB      │ 792 KB        │ 792 KB        │ 792 KB        │         │
├─ starred_command        836.4 µs      │ 1.635 ms      │ 1.01 ms       │ 1.054 ms      │ 100     │ 100
│                         alloc:        │               │               │               │         │
│                           18          │ 18            │ 18            │ 18            │         │
│                           241.2 KB    │ 241.2 KB      │ 241.2 KB      │ 241.2 KB      │         │
│                         dealloc:      │               │               │               │         │
│                           7           │ 7             │ 7             │ 7             │         │
│                           399.7 KB    │ 399.7 KB      │ 399.7 KB      │ 399.7 KB      │         │
│                         grow:         │               │               │               │         │
│                           16          │ 16            │ 16            │ 16            │         │
│                           398.8 KB    │ 398.8 KB      │ 398.8 KB      │ 398.8 KB      │         │
╰─ starred_command_macro  927.8 µs      │ 1.705 ms      │ 1.01 ms       │ 1.059 ms      │ 100     │ 100
                          alloc:        │               │               │               │         │
                            63          │ 63            │ 63            │ 63            │         │
                            244.5 KB    │ 244.5 KB      │ 244.5 KB      │ 244.5 KB      │         │
                          dealloc:      │               │               │               │         │
                            52          │ 52            │ 52            │ 52            │         │
                            404.1 KB    │ 404.1 KB      │ 404.1 KB      │ 404.1 KB      │         │
                          grow:         │               │               │               │         │
                            19          │ 19            │ 19            │ 19            │         │
                            400 KB      │ 400 KB        │ 400 KB        │ 400 KB        │         │
 */
