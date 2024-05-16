import "./style.css";
import "./loader.mjs";
import van, { State } from "vanjs-core";
import { convert_math } from "mitex-wasm";
const { div, textarea, button } = van.tags;

let $typst = window.$typst;

const isDarkMode = () =>
  window.matchMedia?.("(prefers-color-scheme: dark)").matches;

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

  /// The latex code input
  const input = van.state(srcDefault),
    /// The converted Typst code
    output = van.state(""),
    /// The source code state
    error = van.state(""),
    /// The dark mode style
    darkModeStyle = van.derive(() => {
      if (isDarkMode()) {
        return `#set text(fill: rgb("#fff"));`;
      } else {
        return `#set text(fill: rgb("#000"));`;
      }
    });

  /// Drive src, output and error from input
  van.derive(() => {
    try {
      let convert_res = convert_math(input.val, new Uint8Array());
      output.val = convert_res;
      error.val = "";
    } catch (e) {
      output.val = "";
      error.val = e as string;
    }
  });

  /// The preview component
  const Preview = (output: State<string>) => {
    const svgData = van.state("");
    van.derive(async () => {
      if (fontLoaded.val) {
        svgData.val = await $typst.svg({
          mainContent: `#import "@preview/mitex:0.2.4": *
        #set page(width: auto, height: auto, margin: 1em);
        #set text(size: 24pt);
        ${darkModeStyle.val}
        #math.equation(eval("$" + \`${output.val}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)
        `,
        });
      } else {
        svgData.val = "";
      }
    });

    return div(
      { class: "mitex-preview" },
      div({
        innerHTML: van.derive(() => {
          if (!compilerLoaded.val) {
            return "Loading compiler from CDN...";
          } else if (!fontLoaded.val) {
            return "Loading fonts from CDN...";
          } else {
            return svgData.val;
          }
        }),
      })
    );
  };

  /// Copy a a derived string to clipboard
  const CopyButton = (title: string, content: State<string>) =>
    button({
      onclick: () => navigator.clipboard.writeText(content.val),
      textContent: title,
    });

  return div(
    { class: "mitex-main flex-column" },
    div(
      { class: "mitex-edit-row flex-row" },
      textarea({
        class: "mitex-input",
        placeholder: "Type LaTeX math equations here",
        value: srcDefault,
        autofocus: true,
        rows: 10,
        oninput(event: Event) {
          input.val = (event.target! as HTMLInputElement).value;
        },
      }),
      textarea({
        class: "mitex-output",
        value: output,
        readOnly: true,
        placeholder: "Output",
        rows: 10,
        onfocus: (event: Event) =>
          (event.target! as HTMLTextAreaElement).select(),
      })
    ),
    /// Create DOM elements
    CopyButton(
      "Copy with template",
      van.derive(
        () =>
          `#math.equation(eval("$" + \`${output.val}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)`
      )
    ),
    CopyButton(
      "Copy with template and imports",
      van.derive(
        () => `#import "@preview/mitex:0.2.4": *\n
#math.equation(eval("$" + \`${output.val}\`.text + "$", mode: "markup", scope: mitex-scope), block: true)`
      )
    ),
    Preview(output),
    div({ class: "error", textContent: error })
  );
};

van.add(document.querySelector("#app")!, App());
