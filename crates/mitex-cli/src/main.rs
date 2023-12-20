use serde::{Deserialize, Serialize};

fn main() {
    // typst query --root . .\packages\latex-spec\mod.typ "<mitex-packages>"
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = std::path::Path::new(&project_root)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let target_dir = project_root.join("target/mitex-artifacts");

    let package_specs = std::process::Command::new("typst")
        .args([
            "query",
            "--root",
            project_root.to_str().unwrap(),
            project_root
                .join("packages/latex-spec/mod.typ")
                .to_str()
                .unwrap(),
            "<mitex-packages>",
        ])
        .output()
        .expect("failed to query metadata");

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    struct QueryItem<T> {
        pub value: T,
    }

    type Json<T> = Vec<QueryItem<T>>;

    let mut json_spec: mitex_spec::JsonCommandSpec = Default::default();
    let json_packages: Json<mitex_spec::query::PackagesVec> =
        serde_json::from_slice(&package_specs.stdout).expect("failed to parse package specs");
    if json_packages.is_empty() {
        panic!("no package found");
    }
    if json_packages.len() > 1 {
        panic!("multiple packages found");
    }

    std::fs::create_dir_all(target_dir.join("spec")).unwrap();

    let json_packages = json_packages.into_iter().next().unwrap().value;
    std::fs::write(
        target_dir.join("spec/packages.json"),
        serde_json::to_string_pretty(&json_packages).unwrap(),
    )
    .unwrap();

    for package in json_packages.0 {
        for (name, item) in package.spec.commands {
            json_spec.commands.insert(name, item);
        }
    }
    std::fs::write(
        target_dir.join("spec/default.json"),
        serde_json::to_string_pretty(&json_spec).unwrap(),
    )
    .unwrap();

    let spec: mitex_spec::CommandSpec = json_spec.into();

    std::fs::write(target_dir.join("spec/default.rkyv"), spec.to_bytes()).unwrap();
}
