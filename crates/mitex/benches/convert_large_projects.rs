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

last^1 (typst v0.10.0)
Benchmark 1: typst compile --root . crates\mitex\benches\empty.typ
  Time (mean ± σ):     379.0 ms ±   8.8 ms    [User: 101.2 ms, System: 32.8 ms]
  Range (min … max):   369.9 ms … 396.6 ms    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki.typ
  Time (mean ± σ):      2.214 s ±  0.073 s    [User: 0.469 s, System: 0.031 s]
  Range (min … max):    2.096 s …  2.316 s    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki-with-render.typ
  Time (mean ± σ):      3.772 s ±  0.088 s    [User: 1.165 s, System: 0.102 s]
  Range (min … max):    3.591 s …  3.897 s    10 runs

convert_large_projects    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ oiwiki_231222        80.06 ms      │ 96.41 ms      │ 82.89 ms      │ 83.77 ms      │ 100     │ 100
│                       alloc:        │               │               │               │         │
│                         1364697     │ 1364697       │ 1364697       │ 1364697       │         │
│                         85.01 MB    │ 85.01 MB      │ 85.01 MB      │ 85.01 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         1364697     │ 1364697       │ 1364697       │ 1364697       │         │
│                         93.78 MB    │ 93.78 MB      │ 93.78 MB      │ 93.78 MB      │         │
│                       grow:         │               │               │               │         │
│                         64615       │ 64615         │ 64615         │ 64615         │         │
│                         8.763 MB    │ 8.763 MB      │ 8.763 MB      │ 8.763 MB      │         │
╰─ oiwiki_231222_macro  101.8 ms      │ 123.5 ms      │ 104 ms        │ 104.7 ms      │ 100     │ 100
                        alloc:        │               │               │               │         │
                          1527197     │ 1527197       │ 1527197       │ 1527197       │         │
                          155.6 MB    │ 155.6 MB      │ 155.6 MB      │ 155.6 MB      │         │
                        dealloc:      │               │               │               │         │
                          1527197     │ 1527197       │ 1527197       │ 1527197       │         │
                          193.4 MB    │ 193.4 MB      │ 193.4 MB      │ 193.4 MB      │         │
                        grow:         │               │               │               │         │
                          162115      │ 162115        │ 162115        │ 162115        │         │
                          37.88 MB    │ 37.88 MB      │ 37.88 MB      │ 37.88 MB      │         │
 */
