use divan::{AllocProfiler, Bencher};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

fn bench<const WITH_MACRO: bool>(bencher: Bencher, path: &str) {
    // typst query --root . .\packages\latex-spec\mod.typ "<mitex-packages>"
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = std::path::Path::new(&project_root)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let Ok(v) = std::fs::read_to_string(project_root.join(path)) else {
        eprintln!("Cannot read file {}", path);
        return;
    };

    #[derive(serde::Deserialize)]
    struct Fixture {
        text: String,
    }

    let data = serde_json::from_str::<Vec<Fixture>>(&v).unwrap();

    let convert = if WITH_MACRO {
        mitex::convert_math
    } else {
        mitex::convert_math_no_macro
    };

    // warm up
    convert("$ $", None).unwrap();

    bencher.bench(|| {
        for fixture in &data {
            convert(&fixture.text, None).unwrap();
        }
    });
}

#[divan::bench]
fn oiwiki_231222(bencher: Bencher) {
    bench::<false>(bencher, "local/oiwiki-231222.json");
}

#[divan::bench]
fn oiwiki_231222_macro(bencher: Bencher) {
    bench::<true>(bencher, "local/oiwiki-231222.json");
}

/*

last^1 (macro support, typst v0.10.0)
Benchmark 1: typst compile --root . crates\mitex\benches\empty.typ
  Time (mean ± σ):     399.6 ms ±  36.6 ms    [User: 75.0 ms, System: 26.6 ms]
  Range (min … max):   371.0 ms … 495.5 ms    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki.typ
  Time (mean ± σ):      2.179 s ±  0.029 s    [User: 0.681 s, System: 0.049 s]
  Range (min … max):    2.142 s …  2.229 s    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki-with-render.typ
  Time (mean ± σ):      3.830 s ±  0.021 s    [User: 1.223 s, System: 0.168 s]
  Range (min … max):    3.787 s …  3.867 s    10 runs

convert_large_projects  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ oiwiki_231222        79.07 ms      │ 87.49 ms      │ 80.94 ms      │ 81.26 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         1361478     │ 1361478       │ 1361478       │ 1361478       │         │
│                         84.71 MB    │ 84.71 MB      │ 84.71 MB      │ 84.71 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         1361478     │ 1361478       │ 1361478       │ 1361478       │         │
│                         93.27 MB    │ 93.27 MB      │ 93.27 MB      │ 93.27 MB      │         │
│                       grow:         │               │               │               │         │
│                         64098       │ 64098         │ 64098         │ 64098         │         │
│                         8.556 MB    │ 8.556 MB      │ 8.556 MB      │ 8.556 MB      │         │
╰─ oiwiki_231222_macro  79.46 ms      │ 140.6 ms      │ 84.31 ms      │ 85.6 ms       │ 100     │ 100
                        alloc:        │               │               │               │         │
                          1361478     │ 1361478       │ 1361478       │ 1361478       │         │
                          84.71 MB    │ 84.71 MB      │ 84.71 MB      │ 84.71 MB      │         │
                        dealloc:      │               │               │               │         │
                          1361478     │ 1361478       │ 1361478       │ 1361478       │         │
                          93.27 MB    │ 93.27 MB      │ 93.27 MB      │ 93.27 MB      │         │
                        grow:         │               │               │               │         │
                          64098       │ 64098         │ 64098         │ 64098         │         │
                          8.556 MB    │ 8.556 MB      │ 8.556 MB      │ 8.556 MB      │         │

baseline (typst v0.10.0)
Benchmark 1: typst compile --root . crates\mitex\benches\empty.typ
  Time (mean ± σ):     379.0 ms ±   8.8 ms    [User: 101.2 ms, System: 32.8 ms]
  Range (min … max):   369.9 ms … 396.6 ms    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki.typ
  Time (mean ± σ):      2.214 s ±  0.073 s    [User: 0.469 s, System: 0.031 s]
  Range (min … max):    2.096 s …  2.316 s    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki-with-render.typ
  Time (mean ± σ):      3.772 s ±  0.088 s    [User: 1.165 s, System: 0.102 s]
  Range (min … max):    3.591 s …  3.897 s    10 runs

convert_large_projects  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ oiwiki_231222        80.34 ms      │ 86.19 ms      │ 82.85 ms      │ 82.86 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         1388435     │ 1388435       │ 1388435       │ 1388435       │         │
│                         84.32 MB    │ 84.32 MB      │ 84.32 MB      │ 84.32 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         1388435     │ 1388435       │ 1388435       │ 1388435       │         │
│                         94.21 MB    │ 94.21 MB      │ 94.21 MB      │ 94.21 MB      │         │
│                       grow:         │               │               │               │         │
│                         71604       │ 71604         │ 71604         │ 71604         │         │
│                         9.881 MB    │ 9.881 MB      │ 9.881 MB      │ 9.881 MB      │         │
╰─ oiwiki_231222_macro  80.75 ms      │ 88.61 ms      │ 83.29 ms      │ 83.37 ms      │ 100     │ 100
                        alloc:        │               │               │               │         │
                          1388435     │ 1388435       │ 1388435       │ 1388435       │         │
                          84.32 MB    │ 84.32 MB      │ 84.32 MB      │ 84.32 MB      │         │
                        dealloc:      │               │               │               │         │
                          1388435     │ 1388435       │ 1388435       │ 1388435       │         │
                          94.21 MB    │ 94.21 MB      │ 94.21 MB      │ 94.21 MB      │         │
                        grow:         │               │               │               │         │
                          71604       │ 71604         │ 71604         │ 71604         │         │
                          9.881 MB    │ 9.881 MB      │ 9.881 MB      │ 9.881 MB      │         │
 */
