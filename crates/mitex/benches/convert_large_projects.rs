use divan::{AllocProfiler, Bencher};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn oiwiki_231222(bencher: Bencher) {
    // typst query --root . .\packages\latex-spec\mod.typ "<mitex-packages>"
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = std::path::Path::new(&project_root)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let v = std::fs::read_to_string(project_root.join("local/oiwiki-231222.json")).unwrap();

    #[derive(serde::Deserialize)]
    struct Fixture {
        text: String,
    }

    let data = serde_json::from_str::<Vec<Fixture>>(&v).unwrap();

    // warm up
    mitex::convert_math("$ $", None).unwrap();

    bencher.bench(|| {
        for fixture in &data {
            mitex::convert_math(&fixture.text, None).unwrap();
        }
    });
}

/*

last^1 (typst v0.10.0)
Benchmark 1: typst compile --root . crates\mitex\benches\empty.typ
  Time (mean ± σ):     392.1 ms ±  18.3 ms    [User: 82.8 ms, System: 18.8 ms]
  Range (min … max):   372.5 ms … 428.1 ms    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki.typ
  Time (mean ± σ):      2.282 s ±  0.034 s    [User: 0.714 s, System: 0.044 s]
  Range (min … max):    2.228 s …  2.314 s    10 runs
Benchmark 1: typst compile --root . crates\mitex\benches\oiwiki-with-render.typ
  Time (mean ± σ):      3.847 s ±  0.076 s    [User: 1.092 s, System: 0.084 s]
  Range (min … max):    3.747 s …  3.986 s    10 runs

convert_large_projects    fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ oiwiki_231222  82.62 ms      │ 93.44 ms      │ 85.1 ms       │ 85.24 ms      │ 100     │ 100
                  alloc:        │               │               │               │         │
                    1364717     │ 1364717       │ 1364717       │ 1364717       │         │
                    85.01 MB    │ 85.01 MB      │ 85.01 MB      │ 85.01 MB      │         │
                  dealloc:      │               │               │               │         │
                    1364717     │ 1364717       │ 1364717       │ 1364717       │         │
                    93.76 MB    │ 93.76 MB      │ 93.76 MB      │ 93.76 MB      │         │
                  grow:         │               │               │               │         │
                    64578       │ 64578         │ 64578         │ 64578         │         │
                    8.756 MB    │ 8.756 MB      │ 8.756 MB      │ 8.756 MB      │         │
 */
