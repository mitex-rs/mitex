use divan::{AllocProfiler, Bencher};
use mitex_parser::{parse, CommandSpec};

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
        builder.add_command(&format!("alpha{}", i), TEX_SYMBOL);
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

#[divan::bench]
fn alpha_x_20000(bencher: Bencher) {
    bench(bencher, &ALPHA_X_20000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000(bencher: Bencher) {
    bench(bencher, &ALPHA_X_40000, DEFAULT_SPEC.clone());
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

static STARRED_COMMAND: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\operatorname*{a}".repeat(5000));

#[divan::bench]
fn starred_command(bencher: Bencher) {
    bench(bencher, &STARRED_COMMAND, DEFAULT_SPEC.clone());
}

/*
last^1
simple                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ alpha_x_20000        2.198 ms      │ 3.16 ms       │ 2.355 ms      │ 2.44 ms       │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         12          │ 12            │ 12            │ 12            │         │
│                         960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                       dealloc:      │               │               │               │         │
│                         5           │ 5             │ 5             │ 5             │         │
│                         1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                       grow:         │               │               │               │         │
│                         18          │ 18            │ 18            │ 18            │         │
│                         1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ alpha_x_20000_heavy  2.215 ms      │ 3.235 ms      │ 2.394 ms      │ 2.435 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         12          │ 12            │ 12            │ 12            │         │
│                         960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                       dealloc:      │               │               │               │         │
│                         5           │ 5             │ 5             │ 5             │         │
│                         1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                       grow:         │               │               │               │         │
│                         18          │ 18            │ 18            │ 18            │         │
│                         1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ alpha_x_40000        4.605 ms      │ 7.2 ms        │ 4.851 ms      │ 4.969 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         12          │ 12            │ 12            │ 12            │         │
│                         1.92 MB     │ 1.92 MB       │ 1.92 MB       │ 1.92 MB       │         │
│                       dealloc:      │               │               │               │         │
│                         5           │ 5             │ 5             │ 5             │         │
│                         3.152 MB    │ 3.152 MB      │ 3.152 MB      │ 3.152 MB      │         │
│                       grow:         │               │               │               │         │
│                         19          │ 19            │ 19            │ 19            │         │
│                         3.151 MB    │ 3.151 MB      │ 3.151 MB      │ 3.151 MB      │         │
├─ alpha_x_40000_heavy  4.594 ms      │ 6.76 ms       │ 4.918 ms      │ 5.035 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         12          │ 12            │ 12            │ 12            │         │
│                         1.92 MB     │ 1.92 MB       │ 1.92 MB       │ 1.92 MB       │         │
│                       dealloc:      │               │               │               │         │
│                         5           │ 5             │ 5             │ 5             │         │
│                         3.152 MB    │ 3.152 MB      │ 3.152 MB      │ 3.152 MB      │         │
│                       grow:         │               │               │               │         │
│                         19          │ 19            │ 19            │ 19            │         │
│                         3.151 MB    │ 3.151 MB      │ 3.151 MB      │ 3.151 MB      │         │
├─ slice_word           2.696 ms      │ 4.247 ms      │ 3.024 ms      │ 3.09 ms       │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         12          │ 12            │ 12            │ 12            │         │
│                         960.9 KB    │ 960.9 KB      │ 960.9 KB      │ 960.9 KB      │         │
│                       dealloc:      │               │               │               │         │
│                         5           │ 5             │ 5             │ 5             │         │
│                         1.579 MB    │ 1.579 MB      │ 1.579 MB      │ 1.579 MB      │         │
│                       grow:         │               │               │               │         │
│                         18          │ 18            │ 18            │ 18            │         │
│                         1.578 MB    │ 1.578 MB      │ 1.578 MB      │ 1.578 MB      │         │
├─ sqrt_pattern         2.07 ms       │ 4.043 ms      │ 2.28 ms       │ 2.384 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         7523        │ 7523          │ 7523          │ 7523          │         │
│                         1.021 MB    │ 1.021 MB      │ 1.021 MB      │ 1.021 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         8           │ 8             │ 8             │ 8             │         │
│                         793 KB      │ 793 KB        │ 793 KB        │ 793 KB        │         │
│                       grow:         │               │               │               │         │
│                         17          │ 17            │ 17            │ 17            │         │
│                         792 KB      │ 792 KB        │ 792 KB        │ 792 KB        │         │
╰─ starred_command      1.035 ms      │ 4.854 ms      │ 1.304 ms      │ 1.54 ms       │ 100     │ 100
                        alloc:        │               │               │               │         │
                          18          │ 18            │ 18            │ 18            │         │
                          241.2 KB    │ 241.2 KB      │ 241.2 KB      │ 241.2 KB      │         │
                        dealloc:      │               │               │               │         │
                          7           │ 7             │ 7             │ 7             │         │
                          399.7 KB    │ 399.7 KB      │ 399.7 KB      │ 399.7 KB      │         │
                        grow:         │               │               │               │         │
                          16          │ 16            │ 16            │ 16            │         │
                          398.8 KB    │ 398.8 KB      │ 398.8 KB      │ 398.8 KB      │         │
 */
