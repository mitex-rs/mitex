//! Common build script for crates depending on the compacted default
//! specification.

use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

fn main() {
    let spec_builder = if cfg!(feature = "prebuilt") {
        copy_prebuilt
    } else if cfg!(feature = "generate") || which::which("typst").is_ok() {
        generate
    } else {
        // fallback to prebuilt spec
        copy_prebuilt
    };

    spec_builder()
        .with_context(|| "failed to build spec")
        .unwrap();
}

fn get_project_root() -> anyhow::Result<PathBuf> {
    let project_root =
        std::env::var("CARGO_MANIFEST_DIR").with_context(|| "failed to get project root")?;
    Ok(std::path::Path::new(&project_root)
        .parent()
        .with_context(|| "failed to get project root")?
        .parent()
        .with_context(|| "failed to get project root")?
        .to_owned())
}

fn copy_prebuilt() -> anyhow::Result<()> {
    // println!("cargo:warning=copy_prebuilt");
    let project_root = get_project_root()?;

    // assets/artifacts/spec/default.rkyv
    std::fs::create_dir_all(project_root.join("target/mitex-artifacts/spec"))
        .with_context(|| "failed to create target_dir for store spec")?;

    let prebuilt_spec = project_root.join("assets/artifacts/spec/default.rkyv");
    let target_spec = project_root.join("target/mitex-artifacts/spec/default.rkyv");

    std::fs::copy(prebuilt_spec, target_spec).with_context(|| {
        "failed to copy prebuilt spec, \
    do you forget to run `git submodule update --init`?"
    })?;

    Ok(())
}

fn generate() -> anyhow::Result<()> {
    // println!("cargo:warning=generate");

    // require rustc 1.70.0
    println!("cargo:rerun-if-changed=ALWAYS_RUN_ME");

    // typst query --root . ./packages/latex-spec/mod.typ "<mitex-packages>"
    let project_root = get_project_root()?;

    let target_dir = project_root.join("target/mitex-artifacts");

    let package_specs = std::process::Command::new("typst")
        .args([
            "query",
            "--root",
            project_root.to_str().unwrap(),
            project_root
                .join("packages/mitex/specs/mod.typ")
                .to_str()
                .unwrap(),
            "<mitex-packages>",
        ])
        .output()
        .with_context(|| "failed to query metadata")?;

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

    std::fs::create_dir_all(target_dir.join("spec"))
        .with_context(|| "failed to create target_dir for store spec")?;

    let json_packages = json_packages.into_iter().next().unwrap().value;
    std::fs::write(
        target_dir.join("spec/packages.json"),
        serde_json::to_string_pretty(&json_packages)
            .with_context(|| "failed to serialize json packages")?,
    )
    .with_context(|| "failed to write json packages")?;

    for package in json_packages.0 {
        for (name, item) in package.spec.commands {
            json_spec.commands.insert(name, item);
        }
    }
    std::fs::write(
        target_dir.join("spec/default.json"),
        serde_json::to_string_pretty(&json_spec)
            .with_context(|| "failed to serialize json spec")?,
    )
    .with_context(|| "failed to write json spec")?;

    let spec: mitex_spec::CommandSpec = json_spec.into();

    std::fs::write(target_dir.join("spec/default.rkyv"), spec.to_bytes())
        .with_context(|| "failed to write compacted spec")?;

    Ok(())
}
