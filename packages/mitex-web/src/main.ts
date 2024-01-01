import "./style.css";
import van, { State } from "vanjs-core";
import { convert_math } from "mitex-wasm";
const { div, textarea, button } = van.tags;

let $typst = window.$typst;

const App = () => {
  /// Default source code
  const srcDefault = `\\newcommand{\\f}[2]{#1f(#2)}
  \\f\\relax{x} = \\int_{-\\infty}^\\infty
    \\f\\hat\\xi\\,e^{2 \\pi i \\xi x}
    \\,d\\xi`;

  /// Capture compiler load status
  let compilerLoaded = van.state(false);
  let fontLoaded = van.state(false);
  window.$typst$script.then(async () => {
    compilerLoaded.val = true;
    await $typst.svg({ mainContent: "" });
    fontLoaded.val = true;
  });

  /// Create DOM elements
  const input_area = textarea({
    class: "mitex-input",
    placeholder: "Type LaTeX math equations here",
    value: srcDefault,
    autofocus: true,
    rows: 10,
  });
  const copy_template_button = button("Copy with template");
  const copy_template_with_imports_button = button(
    "Copy with template and imports"
  );
  const output = textarea({
    class: "mitex-output",
    readOnly: true,
    placeholder: "Output",
    rows: 10,
  });

  /// The preview component
  const Preview = (sourceCode: State<string>) => {
    const svgData = van.state("");
    console.log(sourceCode);
    van.derive(async () => {
      if (fontLoaded.val) {
        svgData.val = await $typst.svg({ mainContent: sourceCode.val });
      } else {
        svgData.val = "";
      }
    });

    const content = van.derive(() => {
      if (!compilerLoaded.val) {
        return "Loading compiler from CDN...";
      } else if (!fontLoaded.val) {
        return "Loading fonts from CDN...";
      } else {
        return svgData.val;
      }
    });

    console.log(compilerLoaded);
    return div(
      { class: "mitex-preview" },
      div({
        innerHTML: content,
      })
    );
  };

  const previewTmpl = (out: string) => `#import "@preview/mitex:0.1.0": *
#set page(width: auto, height: auto, margin: 1em);
#set text(size: 24pt);
#math.equation(eval("$" + \`${out}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)
`;

  /// The source code state
  const src = van.state(
    previewTmpl(convert_math(srcDefault, new Uint8Array()))
  );

  const error = div({ class: "error" });
  const updateOutput = () => {
    try {
      let convert_res = convert_math(input_area.value, new Uint8Array());
      src.val = previewTmpl(convert_res);
      output.value = convert_res;
      error.textContent = "";
    } catch (e) {
      output.value = "";
      error.textContent = e as string;
    }
  };
  output.onfocus = () => output.select();
  input_area.oninput = updateOutput;
  copy_template_button.onclick = () => {
    updateOutput();
    const res = `#math.equation(eval("$" + \`${output.value}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)`;
    navigator.clipboard.writeText(res);
  };
  copy_template_with_imports_button.onclick = () => {
    updateOutput();
    const res = `#import "@preview/mitex:0.1.0": *

#math.equation(eval("$" + \`${output.value}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)`;
    navigator.clipboard.writeText(res);
  };
  updateOutput();
  return div(
    { class: "mitex-main flex-column" },
    div({ class: "mitex-edit-row flex-row" }, input_area, output),
    copy_template_button,
    copy_template_with_imports_button,
    Preview(src),
    error
  );
};

van.add(document.querySelector("#app")!, App());
