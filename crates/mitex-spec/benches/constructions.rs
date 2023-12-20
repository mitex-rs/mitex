use divan::{AllocProfiler, Bencher};
use mitex_spec::CommandSpec;

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
        builder.add_command(&format!("alpha{}", i), TEX_SYMBOL);
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
        let _ = unsafe { CommandSpec::from_bytes_unchecked(&bytes) };
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
constructions                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ deserialize_1               199.8 ns      │ 11.09 µs      │ 299.8 ns      │ 397.8 ns      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
│                              dealloc:      │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
├─ deserialize_1000            107.7 µs      │ 596.8 µs      │ 117 µs        │ 124.2 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
├─ deserialize_100000          17.45 ms      │ 29.96 ms      │ 20.27 ms      │ 20.76 ms      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
│                              dealloc:      │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
├─ deserialize_trusted_1       143.6 ns      │ 295.2 ns      │ 149.8 ns      │ 169 ns        │ 100     │ 6400
│                              alloc:        │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
│                              dealloc:      │               │               │               │         │
│                                3           │ 3             │ 3             │ 3             │         │
│                                410 B       │ 410 B         │ 410 B         │ 410 B         │         │
├─ deserialize_trusted_1000    80.59 µs      │ 568.8 µs      │ 154.1 µs      │ 163.1 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                1002        │ 1002          │ 1002          │ 1002          │         │
│                                173.8 KB    │ 173.8 KB      │ 173.8 KB      │ 173.8 KB      │         │
├─ deserialize_trusted_100000  10.86 ms      │ 20.24 ms      │ 14.07 ms      │ 14.24 ms      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
│                              dealloc:      │               │               │               │         │
│                                100002      │ 100002        │ 100002        │ 100002        │         │
│                                11.6 MB     │ 11.6 MB       │ 11.6 MB       │ 11.6 MB       │         │
├─ prelude_1                   193.6 ns      │ 226.4 ns      │ 198.3 ns      │ 203.5 ns      │ 100     │ 6400
│                              alloc:        │               │               │               │         │
│                                4           │ 4             │ 4             │ 4             │         │
│                                420 B       │ 420 B         │ 420 B         │ 420 B         │         │
│                              dealloc:      │               │               │               │         │
│                                4           │ 4             │ 4             │ 4             │         │
│                                420 B       │ 420 B         │ 420 B         │ 420 B         │         │
├─ prelude_1000                182.7 µs      │ 2.092 ms      │ 216.3 µs      │ 250.5 µs      │ 100     │ 100
│                              alloc:        │               │               │               │         │
│                                2011        │ 2011          │ 2011          │ 2011          │         │
│                                349.5 KB    │ 349.5 KB      │ 349.5 KB      │ 349.5 KB      │         │
│                              dealloc:      │               │               │               │         │
│                                2011        │ 2011          │ 2011          │ 2011          │         │
│                                349.5 KB    │ 349.5 KB      │ 349.5 KB      │ 349.5 KB      │         │
╰─ prelude_100000              20.84 ms      │ 33.22 ms      │ 24.95 ms      │ 25.48 ms      │ 100     │ 100
                               alloc:        │               │               │               │         │
                                 200017      │ 200017        │ 200017        │ 200017        │         │
                                 23.22 MB    │ 23.22 MB      │ 23.22 MB      │ 23.22 MB      │         │
                               dealloc:      │               │               │               │         │
                                 200017      │ 200017        │ 200017        │ 200017        │         │
                                 23.22 MB    │ 23.22 MB      │ 23.22 MB      │ 23.22 MB      │         │
 */
