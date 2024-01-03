import "./underleaf.css";
import "./typst.css";
import "./typst.ts";
import "../loader.mjs";
import van from "vanjs-core";
const { div, button } = van.tags;

import { Editor } from "./underleaf-editor";
import { DirectoryView, FsItemState } from "./underleaf-fs";
import { Preview } from "./underleaf-preview";
import { TypstDocument } from "typst-dom/typst-doc.mjs";
import { IncrementalServer } from "@myriaddreamin/typst.ts/dist/esm/compiler.mjs";

let $typst = window.$typst;

/// Checks if the browser is in dark mode
const isDarkMode = () =>
  window.matchMedia?.("(prefers-color-scheme: dark)").matches;

/// Exports the document
const ExportButton = (title: string, onclick: () => void) =>
  button({
    onclick,
    textContent: title,
  });

const App = () => {
  /// External status
  const /// Captures compiler load status
    compilerLoaded = van.state(false),
    /// Captures font load status
    fontLoaded = van.state(false),
    /// Binds to filesystem reload event
    reloadBell = van.state(false);

  const mainFilePath = "/repo/fixtures/underleaf/ieee/main.typ";

  /// Component status
  const /// The incremental server
    incrServer = van.state<IncrementalServer | undefined>(undefined),
    /// The document in memory
    typstDoc = van.state<TypstDocument | undefined>(undefined),
    /// request to change focus file
    changeFocusFile = van.state<FsItemState | undefined>(undefined),
    /// The current focus file
    focusFile = van.state<FsItemState | undefined>(undefined);

  /// Styles and outputs
  const /// The source code state
    error = van.state(""),
    /// The dark mode style
    darkMode = van.state(isDarkMode());

  /// Checks compiler status
  window.$typst$script.then(async () => {
    $typst = window.$typst;

    await $typst.getCompiler();
    compilerLoaded.val = true;
    await $typst.svg({ mainContent: "" });
    fontLoaded.val = true;

    (await $typst.getCompiler()).withIncrementalServer(
      (srv: IncrementalServer) => {
        return new Promise((disposeServer) => {
          incrServer.val = srv;

          /// Let it leak.
          void disposeServer;
        });
      }
    );
  });

  /// Listens to dark mode change
  window
    .matchMedia?.("(prefers-color-scheme: dark)")
    .addEventListener("change", (event) => (darkMode.val = event.matches));

  /// Triggers compilation when precondition is met or changed
  van.derive(async () => {
    try {
      if (
        /// Compiler with fonts should be loaded
        fontLoaded.val &&
        /// Incremental server should be loaded
        incrServer.val &&
        /// Typst document should be loaded
        typstDoc.val &&
        /// Filesystem should be loaded
        reloadBell.val &&
        /// recompile If focus file changed
        focusFile.val &&
        /// recompile If focus file content changed
        focusFile.val.data.val &&
        /// recompile If dark mode changed
        darkMode
      ) {
        console.log("recompilation");

        setTypstTheme(darkMode.val);

        const v = await (
          await $typst.getCompiler()
        ).compile({
          mainFilePath,
          incrementalServer: incrServer.val,
        });

        // todo: incremental update
        typstDoc.val.addChangement(["diff-v1", v as any]);
      }

      error.val = "";
    } catch (e) {
      error.val = e as string;
    }
  });

  const exportAs = (data: string | Uint8Array, mime: string) => {
    var fileBlob = new Blob([data], { type: mime });

    // Create element with <a> tag
    const link = document.createElement("a");

    // name
    link.download =
      mime === "application/pdf"
        ? "A typesetting system to untangle the scientific writing process.pdf"
        : "A typesetting system to untangle the scientific writing process.html";

    // Add file content in the object URL
    link.href = URL.createObjectURL(fileBlob);

    // Add file name
    link.target = "_blank";

    // Add click event to <a> tag to save file.
    link.click();
    URL.revokeObjectURL(link.href);
  };

  const exportPdf = () => {
    setTypstTheme(false);
    const pdfData = $typst.pdf({ mainFilePath });
    return pdfData.then((pdfData: string) =>
      exportAs(pdfData, "application/pdf")
    );
  };

  const exportHtml = () => {
    setTypstTheme(false);
    const svgData = $typst.svg({
      mainFilePath,
      data_selection: { body: true, defs: true, css: true },
    });
    return svgData.then((svgData: string) =>
      exportAs(
        `<!DOCTYPE html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>A typesetting system to untangle the scientific writing process</title></head>
<body>${svgData}</body>
</html>
`,
        "text/html"
      )
    );
  };

  return div(
    { class: "mitex-main flex-column" },
    div(
      {
        class: "flex-row",
        style: "justify-content: space-between; margin-bottom: 10px",
      },
      div(
        {
          style:
            "display: flex; align-items: center; text-align: center; text-decoration: underline; padding-left: 10px",
        },
        "A typesetting system to untangle the scientific writing process"
      ),
      div(
        { class: "mitex-toolbar-row flex-row" },
        div({ class: "error", textContent: error }),
        ExportButton("Export to PDF", exportPdf),
        div({ style: "width: 5px" }),
        ExportButton("HTML", exportHtml)
      )
    ),
    div(
      { class: "mitex-edit-row flex-row" },
      DirectoryView({ compilerLoaded, changeFocusFile, focusFile, reloadBell }),
      Editor(darkMode, changeFocusFile, focusFile),
      Preview({ darkMode, compilerLoaded, fontLoaded, typstDoc })
    )
  );

  async function setTypstTheme(darkMode: boolean) {
    let styling = darkMode
      ? `#let prefer-theme = "dark";`
      : `#let prefer-theme = "light";`;
    await $typst.addSource(
      "/repo/fixtures/underleaf/ieee/styling.typ",
      styling
    );
  }
};

van.add(document.querySelector("#app")!, App());
