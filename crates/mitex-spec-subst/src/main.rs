use std::fs;

use anyhow::Context;

fn walrus_config_from_options() -> walrus::ModuleConfig {
    let mut config = walrus::ModuleConfig::new();
    config.generate_producers_section(true);
    config
}

fn main() -> anyhow::Result<()> {
    let path = "typst-package/mitex.wasm";
    let output = "typst-package/mitex-embed.wasm";
    let buf = fs::read(path).with_context(|| format!("failed to read file {}", path))?;

    let config = walrus_config_from_options();
    let mut module = config.parse(&buf)?;
    mitex_spec::embed_spec(&mut module)?;

    module
        .emit_wasm_file(output)
        .with_context(|| format!("failed to emit embeded wasm to {}", output))?;

    Ok(())
}
