//! Simple benchmarks for construction of the spec.
//!
//! The result shows that the performance in order from high to low is:
//! + `deserialize_trusted_*`
//! + `deserialize_*`
//! + `deserialize_json_*`
//!
//! The Json parsing is used for parsing the query result of `typst`.
//! The Rkyv parsing is used for parsing the embedded spec in the binary.
//! The trusted (unsafe) Rkyv parsing is not used yet.

use divan::{AllocProfiler, Bencher};
use mitex_spec::{ArgPattern, ArgShape, CmdShape, CommandSpec};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

fn prelude_n(n: i32) -> CommandSpec {
    use mitex_spec::preludes::command::*;
    let mut builder = SpecBuilder::default();
    for i in 0..n {
        builder.add_command(&format!("alpha{i}"), TEX_SYMBOL);
    }
    builder.build()
}

#[divan::bench]
fn prelude_1() {
    prelude_n(1);
}

#[divan::bench]
fn prelude_1000() {
    prelude_n(1000);
}

#[divan::bench]
fn prelude_100000() {
    prelude_n(100000);
}

fn bench_json_deserialize(bencher: Bencher, n: i32) {
    use mitex_spec::query;
    const JSON_TEX_SYMBOL: query::CommandSpecItem = query::CommandSpecItem::Cmd(CmdShape {
        args: ArgShape::Right {
            pattern: ArgPattern::None,
        },
        alias: None,
    });

    let mut spec = query::CommandSpecRepr::default();
    for i in 0..n {
        spec.commands.insert(format!("alpha{i}"), JSON_TEX_SYMBOL);
    }

    let bytes = serde_json::to_vec(&spec).unwrap();
    let bytes = bytes.as_slice();

    bencher.bench(|| {
        let _: query::CommandSpecRepr = serde_json::from_slice(bytes).unwrap();
    });
}

#[divan::bench]
fn deserialize_json_1(bencher: Bencher) {
    bench_json_deserialize(bencher, 1);
}

#[divan::bench]
fn deserialize_json_1000(bencher: Bencher) {
    bench_json_deserialize(bencher, 1000);
}

#[divan::bench]
fn deserialize_json_100000(bencher: Bencher) {
    bench_json_deserialize(bencher, 100000);
}

#[cfg(feature = "rkyv")]
#[cfg(feature = "rkyv-validation")]
fn bench_deserialize(bencher: Bencher, spec: CommandSpec) {
    let bytes = spec.to_bytes();
    bencher.bench(|| {
        let _ = CommandSpec::from_bytes(&bytes);
    });
}

#[divan::bench]
#[cfg(feature = "rkyv")]
#[cfg(feature = "rkyv-validation")]
fn deserialize_1(bencher: Bencher) {
    bench_deserialize(bencher, prelude_n(1));
}

#[divan::bench]
#[cfg(feature = "rkyv")]
#[cfg(feature = "rkyv-validation")]
fn deserialize_1000(bencher: Bencher) {
    bench_deserialize(bencher, prelude_n(1000));
}

#[divan::bench]
#[cfg(feature = "rkyv")]
#[cfg(feature = "rkyv-validation")]
fn deserialize_100000(bencher: Bencher) {
    bench_deserialize(bencher, prelude_n(100000));
}

#[cfg(feature = "rkyv")]
fn bench_deserialize_trusted(bencher: Bencher, spec: CommandSpec) {
    let bytes = spec.to_bytes();
    bencher.bench(|| {
        let _ = {
            // Safety: the data source is trusted and valid.
            unsafe { CommandSpec::from_bytes_unchecked(&bytes) }
        };
    });
}

#[divan::bench]
#[cfg(feature = "rkyv")]
fn deserialize_trusted_1(bencher: Bencher) {
    bench_deserialize_trusted(bencher, prelude_n(1));
}

#[divan::bench]
#[cfg(feature = "rkyv")]
fn deserialize_trusted_1000(bencher: Bencher) {
    bench_deserialize_trusted(bencher, prelude_n(1000));
}

#[divan::bench]
#[cfg(feature = "rkyv")]
fn deserialize_trusted_100000(bencher: Bencher) {
    bench_deserialize_trusted(bencher, prelude_n(100000));
}

/*
last^1
onstructions                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ deserialize_1               170.2 ns      │ 268.6 ns      │ 185.8 ns      │ 201 ns        │ 100     │ 6400
│                              alloc:        │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
│                              dealloc:      │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
├─ deserialize_1000            110.1 µs      │ 190.4 µs      │ 117.6 µs      │ 120.3 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
├─ deserialize_100000          17.17 ms      │ 27.64 ms      │ 19.4 ms       │ 19.92 ms      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
│                              dealloc:      │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
├─ deserialize_json_1          199.8 ns      │ 12.79 µs      │ 199.8 ns      │ 382.8 ns      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                2           │ 2             │ 2             │ 2             │         │
│                                346 B       │ 346 B         │ 346 B         │ 346 B         │         │
│                              dealloc:      │               │               │               │         │
│                                2           │ 2             │ 2             │ 2             │         │
│                                346 B       │ 346 B         │ 346 B         │ 346 B         │         │
├─ deserialize_json_1000       235.2 µs      │ 626.4 µs      │ 268.5 µs      │ 292.3 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                1010        │ 1010          │ 1010          │ 1010          │         │
│                                339.5 KB    │ 339.5 KB      │ 339.5 KB      │ 339.5 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                1010        │ 1010          │ 1010          │ 1010          │         │
│                                339.5 KB    │ 339.5 KB      │ 339.5 KB      │ 339.5 KB      │         │
├─ deserialize_json_100000     26.02 ms      │ 41.23 ms      │ 30.41 ms      │ 30.67 ms      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                100016      │ 100016        │ 100016        │ 100016        │         │
│                                22.22 MB    │ 22.22 MB      │ 22.22 MB      │ 22.22 MB      │         │
│                              dealloc:      │               │               │               │         │
│                                100016      │ 100016        │ 100016        │ 100016        │         │
│                                22.22 MB    │ 22.22 MB      │ 22.22 MB      │ 22.22 MB      │         │
├─ deserialize_trusted_1       145.9 ns      │ 167.8 ns      │ 154.5 ns      │ 153.3 ns      │ 100     │ 12800
│                              alloc:        │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
│                              dealloc:      │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
├─ deserialize_trusted_1000    64.59 µs      │ 265.3 µs      │ 79.59 µs      │ 86.64 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
├─ deserialize_trusted_100000  11.09 ms      │ 18.9 ms       │ 14.18 ms      │ 14.34 ms      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
│                              dealloc:      │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
├─ prelude_1                   176.4 ns      │ 209.2 ns      │ 188.9 ns      │ 189.7 ns      │ 100     │ 6400
│                              alloc:        │               │               │               │         │
│                                4           │ 4             │ 4             │ 4             │         │
│                                420 B       │ 420 B         │ 420 B         │ 420 B         │         │
│                              dealloc:      │               │               │               │         │
│                                4           │ 4             │ 4             │ 4             │         │
│                                420 B       │ 420 B         │ 420 B         │ 420 B         │         │
├─ prelude_1000                158.3 µs      │ 487.2 µs      │ 223.9 µs      │ 236.5 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                2011        │ 2011          │ 2011          │ 2011          │         │
│                                349.5 KB    │ 349.5 KB      │ 349.5 KB      │ 349.5 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                2011        │ 2011          │ 2011          │ 2011          │         │
│                                349.5 KB    │ 349.5 KB      │ 349.5 KB      │ 349.5 KB      │         │
╰─ prelude_100000              19.85 ms      │ 33.71 ms      │ 24.55 ms      │ 24.93 ms      │ 100     │ 100
                               alloc:        │               │               │               │         │
                                 200017      │ 200017        │ 200017        │ 200017        │         │
                                 23.22 MB    │ 23.22 MB      │ 23.22 MB      │ 23.22 MB      │         │
                               dealloc:      │               │               │               │         │
                                 200017      │ 200017        │ 200017        │ 200017        │         │
                                 23.22 MB    │ 23.22 MB      │ 23.22 MB      │ 23.22 MB      │         │
 */
