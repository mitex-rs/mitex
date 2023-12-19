use mitex_parser::{parse, CommandSpec};

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

#[divan::bench]
fn alpha_x_20000() {
    parse(&ALPHA_X_20000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000() {
    parse(&ALPHA_X_40000, DEFAULT_SPEC.clone());
}

#[divan::bench]
fn alpha_x_20000_heavy() {
    parse(&ALPHA_X_20000, HEAVY_SPEC.clone());
}

#[divan::bench]
fn alpha_x_40000_heavy() {
    parse(&ALPHA_X_40000, HEAVY_SPEC.clone());
}

static SLICE_WORD: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\frac ab ".repeat(20000));

#[divan::bench]
fn slice_word() {
    parse(&SLICE_WORD, DEFAULT_SPEC.clone());
}

static SQRT_PATTERN: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| "\\sqrt[1]{2} \\sqrt{2} \\sqrt{2} \\sqrt{2}".repeat(2500));

#[divan::bench]
fn sqrt_pattern() {
    parse(&SQRT_PATTERN, DEFAULT_SPEC.clone());
}

/*
last^1
simple                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ alpha_x_20000        2.445 ms      │ 3.685 ms      │ 2.689 ms      │ 2.787 ms      │ 100     │ 100
├─ alpha_x_20000_heavy  2.505 ms      │ 3.501 ms      │ 2.683 ms      │ 2.781 ms      │ 100     │ 100
├─ alpha_x_40000        5.122 ms      │ 8.133 ms      │ 5.565 ms      │ 5.699 ms      │ 100     │ 100
├─ alpha_x_40000_heavy  5.142 ms      │ 8.89 ms       │ 5.584 ms      │ 5.724 ms      │ 100     │ 100
├─ slice_word           2.774 ms      │ 4.383 ms      │ 3.054 ms      │ 3.112 ms      │ 100     │ 100
╰─ sqrt_pattern         2.331 ms      │ 5.393 ms      │ 2.511 ms      │ 2.627 ms      │ 100     │ 100
 */
