import van, { State } from "vanjs-core";
import { PreviewMode, TypstDocument } from "typst-dom/typst-doc.mjs";
import type { RenderSession } from "@myriaddreamin/typst.ts/dist/esm/renderer.mjs";

const { div } = van.tags;

export interface PreviewViewState {
  darkMode: State<boolean>;
  compilerLoaded: State<boolean>;
  fontLoaded: State<boolean>;
  typstDoc: State<TypstDocument | undefined>;
}

/// The preview component
export const Preview = ({
  darkMode,
  compilerLoaded,
  fontLoaded,
  typstDoc,
}: PreviewViewState) => {
  const previewRef = van.state<HTMLDivElement | undefined>(undefined);
  const kModule = van.state<RenderSession | undefined>(undefined);

  /// Creates a render session
  van.derive(
    async () =>
      fontLoaded.val &&
      (await window.$typst.getRenderer()).runWithSession(
        (m: RenderSession) /* module kernel from wasm */ => {
          return new Promise(async (kModuleDispose) => {
            kModule.val = m;
            /// simply let session leak
            void kModuleDispose;
          });
        }
      )
  );

  /// Creates a TypstDocument
  van.derive(() => {
    if (!(kModule.val && previewRef.val)) {
      return;
    }

    if (typstDoc.val) {
      return;
    }

    const hookedElem = previewRef.val!;
    if (hookedElem.firstElementChild?.tagName !== "svg") {
      hookedElem.innerHTML = "";
    }
    const resizeTarget = document.getElementById("mitex-preview")!;

    const doc = (typstDoc.val = new TypstDocument(hookedElem, kModule.val!, {
      previewMode: PreviewMode.Doc,
      isContentPreview: false,
      sourceMapping: false,
      // set rescale target to `body`
      retrieveDOMState() {
        return {
          // reserving 1px to hide width border
          width: resizeTarget.clientWidth + 1,
          // reserving 1px to hide width border
          height: resizeTarget.offsetHeight,
          boundingRect: resizeTarget.getBoundingClientRect(),
        };
      },
    }));
    doc.setPartialRendering(true);

    /// Responds to dark mode change
    van.derive(() => doc.setPageColor(darkMode.val ? "#242424" : "white"));
  });

  return div({ id: "mitex-preview" }, (dom?: Element) => {
    dom ||= div();
    if (!compilerLoaded.val) {
      dom.textContent = "Loading compiler from CDN...";
    } else if (!fontLoaded.val) {
      dom.textContent = "Loading fonts from CDN...";
    } else {
      dom.textContent = "";
      /// Catches a new reference to dom
      previewRef.val = dom as HTMLDivElement;
    }
    return dom;
  });
};
