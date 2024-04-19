//! Common build script for crates depending on the compacted default
//! specification.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

fn main() {
    let project_root = get_project_root();

    let spec_builder = if cfg!(feature = "prebuilt") {
        copy_prebuilt
    } else if cfg!(feature = "generate") || (which::which("typst").is_ok() && project_root.is_ok())
    {
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
        std::env::var("CARGO_MANIFEST_DIR").with_context(|| "failed to get manifest dir")?;
    let mut project_root = std::path::Path::new(&project_root);
    Ok(loop {
        let parent = project_root
            .parent()
            .with_context(|| "failed to get project root")?;
        if parent.join("Cargo.toml").exists() {
            break parent.to_owned();
        }
        project_root = parent;
    })
}

fn copy_prebuilt() -> anyhow::Result<()> {
    // println!("cargo:warning=copy_prebuilt");
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").with_context(|| "failed to get manifest dir")?;
    let manifest_dir = std::path::Path::new(&manifest_dir);
    let target_spec =
        Path::new(&std::env::var("OUT_DIR").unwrap()).join("mitex-artifacts/spec/default.rkyv");

    // assets/artifacts/spec/default.rkyv
    std::fs::create_dir_all(
        target_spec
            .parent()
            .context("failed to get dirname of target spec")?,
    )
    .with_context(|| "failed to create target_dir for store spec")?;

    let prebuilt_spec = manifest_dir.join(Path::new("assets/artifacts/spec/default.rkyv"));
    println!("cargo:warning=Use prebuilt spec binaries at {prebuilt_spec:?}");

    std::fs::copy(prebuilt_spec, target_spec).with_context(|| {
        "failed to copy prebuilt spec, \
    do you forget to run `git submodule update --init`?"
    })?;

    Ok(())
}

fn generate() -> anyhow::Result<()> {
    // println!("cargo:warning=generate");

    // typst query --root . ./packages/latex-spec/mod.typ "<mitex-packages>"
    let project_root = get_project_root()?;

    let spec_root = project_root.join("packages/mitex/specs/");
    println!(
        "cargo:rerun-if-changed={spec_root}",
        spec_root = spec_root.display()
    );

    let target_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("mitex-artifacts");

    let mut package_specs = std::process::Command::new("typst");
    let package_specs = package_specs.args([
        "query",
        "--root",
        project_root.to_str().unwrap(),
        project_root
            .join("packages/mitex/specs/mod.typ")
            .to_str()
            .unwrap(),
        "<mitex-packages>",
    ]);

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    struct QueryItem<T> {
        pub value: T,
    }

    type Json<T> = Vec<QueryItem<T>>;

    let mut json_spec: mitex_spec::JsonCommandSpec = Default::default();
    let json_packages: Json<mitex_spec::query::PackagesVec> = serde_json::from_slice(
        &package_specs
            .output()
            .with_context(|| "failed to query metadata")?
            .stdout,
    )
    .context(format!(
        "failed to parse package specs cmd: {package_specs:?}"
    ))?;
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
