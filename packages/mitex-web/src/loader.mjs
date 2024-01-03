const e = `https://cdn.jsdelivr.net/npm`;
const ns = `@myriaddreamin`;
const tsConfig = {
  lib: `${e}/${ns}/typst.ts@v0.4.2-rc4/dist/esm/contrib/all-in-one-lite.bundle.js`,
  compilerModule: `${e}/${ns}/typst-ts-web-compiler@v0.4.2-rc4/pkg/typst_ts_web_compiler_bg.wasm`,
  rendererModule: `${e}/${ns}/typst-ts-renderer@v0.4.2-rc4/pkg/typst_ts_renderer_bg.wasm`,
};
window.$typst$script = new Promise((resolve) => {
  const head = document.getElementsByTagName("head")[0];
  const s = document.createElement("script");
  s.type = "module";
  s.onload = resolve;
  s.src = tsConfig.lib;
  s.id = "typst";
  head.appendChild(s);
}).then(() => {
  const $typst = window.$typst;
  $typst.setCompilerInitOptions({ getModule: () => tsConfig.compilerModule });
  $typst.setRendererInitOptions({ getModule: () => tsConfig.rendererModule });
});