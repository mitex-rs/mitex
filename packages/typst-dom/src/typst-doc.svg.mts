import { PreviewMode } from "./typst-doc.mjs";
import { TypstCancellationToken } from "./typst-cancel.mjs";
import { TypstPatchAttrs, isDummyPatchElem } from "./typst-patch.mjs";
import type { GConstructor, TypstDocumentContext } from "./typst-doc.mjs";
import type { CanvasPage, TypstCanvasDocument } from "./typst-doc.canvas.mjs";
import { patchSvgToContainer } from "./typst-patch.svg.mjs";

export interface TypstSvgDocument {}

export function provideSvgDoc<
  TBase extends GConstructor<
    TypstDocumentContext & Partial<TypstCanvasDocument>
  >
>(Base: TBase): TBase & GConstructor<TypstSvgDocument> {
  return class SvgDocument extends Base {
    constructor(...args: any[]) {
      super(...args);
      this.registerMode("svg");
    }

    shouldMixinCanvas(): this is TypstCanvasDocument {
      return !!this.feat$canvas;
    }

    postRender$svg() {
      const docRoot = this.hookedElem.firstElementChild as SVGElement;
      if (docRoot) {
        window.initTypstSvg(docRoot);
        this.r.rescale();
      }
    }

    rerender$svg() {
      let patchStr: string;
      const mode = this.previewMode;
      if (mode === PreviewMode.Doc) {
        patchStr = this.fetchSvgDataByDocMode();
      } else if (mode === PreviewMode.Slide) {
        patchStr = this.fetchSvgDataBySlideMode();
      } else {
        throw new Error(`unknown preview mode ${mode}`);
      }

      const t2 = performance.now();
      patchSvgToContainer(this.hookedElem, patchStr, (elem) =>
        this.decorateSvgElement(elem, mode)
      );
      const t3 = performance.now();

      return [t2, t3];
    }

    private fetchSvgDataBySlideMode() {
      const pagesInfo = this.kModule.retrievePagesInfo();

      if (pagesInfo.length === 0) {
        // svg warning
        return `<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
  <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" font-size="20">No page found</text>
</svg>`;
      }

      if (this.partialRenderPage >= pagesInfo.length) {
        this.partialRenderPage = pagesInfo.length - 1;
      }

      const pageOffset = this.partialRenderPage;
      let lo = { x: 0, y: 0 },
        hi = { x: 0, y: 0 };
      for (let i = 0; i < pageOffset; i++) {
        const pageInfo = pagesInfo[i];
        lo.y += pageInfo.height;
      }
      const page = pagesInfo[pageOffset];
      hi.y = lo.y + page.height;
      hi.x = page.width;

      console.log("render_in_window for slide mode", lo.x, lo.y, hi.x, hi.y);

      // with a bit padding to avoid edge error
      lo.x += 1e-1;
      lo.y += 1e-1;
      hi.x -= 1e-1;
      hi.y -= 1e-1;

      return this.kModule.renderSvgDiff({
        window: {
          lo,
          hi,
        },
      });
    }

    private fetchSvgDataByDocMode() {
      const { revScale, left, top, width, height } = this.statSvgFromDom();

      let patchStr: string;
      // with 1px padding to avoid edge error
      if (this.partialRendering) {
        /// Adjust top and bottom
        const ch = this.hookedElem.firstElementChild?.children;
        let topEstimate = top - 1,
          bottomEstimate = top + height + 1;
        if (ch) {
          const pages = Array.from(ch).filter((x) =>
            x.classList.contains("typst-page")
          );
          let minTop = 1e33,
            maxBottom = -1e33,
            accumulatedHeight = 0;
          const translateRegex = /translate\(([-0-9.]+), ([-0-9.]+)\)/;
          for (const page of pages) {
            const pageHeight = Number.parseFloat(
              page.getAttribute("data-page-height")!
            );
            const translate = page.getAttribute("transform")!;
            const translateMatch = translate.match(translateRegex)!;
            const translateY = Number.parseFloat(translateMatch[2]);
            if (translateY + pageHeight > topEstimate) {
              minTop = Math.min(minTop, accumulatedHeight);
            }
            if (translateY < bottomEstimate) {
              maxBottom = Math.max(maxBottom, accumulatedHeight + pageHeight);
            }
            accumulatedHeight += pageHeight;
          }

          if (pages.length != 0) {
            topEstimate = minTop;
            bottomEstimate = maxBottom;
          } else {
            topEstimate = 0;
            bottomEstimate = 1e33;
          }
        }
        // translate
        patchStr = this.kModule.render_in_window(
          // lo.x, lo.y
          left - 1,
          topEstimate,
          // hi.x, hi.y
          left + width + 1,
          bottomEstimate
        );
        console.log(
          "render_in_window with partial rendering enabled window",
          revScale,
          left,
          top,
          width,
          height,
          ", patch scale",
          patchStr.length
        );
      } else {
        console.log(
          "render_in_window with partial rendering disabled",
          0,
          0,
          1e33,
          1e33
        );
        patchStr = this.kModule.render_in_window(0, 0, 1e33, 1e33);
      }

      return patchStr;
    }

    private rescaleSvgOn(svg: SVGElement) {
      const scale = this.getSvgScaleRatio();
      if (scale === 0) {
        console.warn("determine scale as 0, skip rescale");
        return;
      }

      // apply scale
      const dataWidth = Number.parseFloat(svg.getAttribute("data-width")!);
      const dataHeight = Number.parseFloat(svg.getAttribute("data-height")!);
      const appliedWidth = (dataWidth * scale).toString();
      const appliedHeight = (dataHeight * scale).toString();
      const scaledWidth = Math.ceil(dataWidth * scale);
      const scaledHeight = Math.ceil(dataHeight * scale);

      // set data applied width and height to memoize change
      if (svg.getAttribute("data-applied-width") !== appliedWidth) {
        svg.setAttribute("data-applied-width", appliedWidth);
        svg.setAttribute("width", `${scaledWidth}`);
      }
      if (svg.getAttribute("data-applied-height") !== appliedHeight) {
        svg.setAttribute("data-applied-height", appliedHeight);
        svg.setAttribute("height", `${scaledHeight}`);
      }
    }

    // Note: one should retrieve dom state before rescale
    rescale$svg() {
      // get dom state from cache, so we are free from layout reflowing
      const svg = this.hookedElem.firstElementChild as SVGElement;
      if (!svg) {
        return;
      }

      const scale = this.getSvgScaleRatio();
      if (scale === 0) {
        console.warn("determine scale as 0, skip rescale");
        return;
      }

      // get dom state from cache, so we are free from layout reflowing
      const container = this.cachedDOMState;

      // apply scale
      const dataWidth = Number.parseFloat(svg.getAttribute("data-width")!);
      const dataHeight = Number.parseFloat(svg.getAttribute("data-height")!);
      const scaledWidth = Math.ceil(dataWidth * scale);
      const scaledHeight = Math.ceil(dataHeight * scale);

      this.rescaleSvgOn(svg);

      const widthAdjust = Math.max((container.width - scaledWidth) / 2, 0);
      let transformAttr = "";
      if (this.previewMode === PreviewMode.Slide) {
        const heightAdjust = Math.max((container.height - scaledHeight) / 2, 0);
        transformAttr = `translate(${widthAdjust}px, ${heightAdjust}px)`;
      } else {
        transformAttr = `translate(${widthAdjust}px, 0px)`;
      }
      if (this.hookedElem.style.transform !== transformAttr) {
        this.hookedElem.style.transform = transformAttr;
      }

      // change height of the container back from `installCtrlWheelHandler` hack
      if (this.hookedElem.style.height) {
        this.hookedElem.style.removeProperty("height");
      }
    }

    private decorateSvgElement(svg: SVGElement, mode: PreviewMode) {
      const container = this.cachedDOMState;
      const kShouldMixinCanvas =
        this.previewMode === PreviewMode.Doc && this.shouldMixinCanvas();

      // the <rect> could only have integer width and height
      // so we scale it by 100 to make it more accurate
      const INNER_RECT_UNIT = 100;
      const INNER_RECT_SCALE = "scale(0.01)";

      /// Caclulate width
      let maxWidth = 0;

      interface SvgPage {
        elem: Element;
        width: number;
        height: number;
        index: number;
      }

      const nextPages: SvgPage[] = (() => {
        /// Retrieve original pages
        const filteredNextPages = Array.from(svg.children).filter((x) =>
          x.classList.contains("typst-page")
        );

        if (mode === PreviewMode.Doc) {
          return filteredNextPages;
        } else if (mode === PreviewMode.Slide) {
          // already fetched pages info
          const pageOffset = this.partialRenderPage;
          return [filteredNextPages[pageOffset]];
        } else {
          throw new Error(`unknown preview mode ${mode}`);
        }
      })().map((elem, index) => {
        const width = Number.parseFloat(elem.getAttribute("data-page-width")!);
        const height = Number.parseFloat(
          elem.getAttribute("data-page-height")!
        );
        maxWidth = Math.max(maxWidth, width);
        return {
          index,
          elem,
          width,
          height,
        };
      });

      /// Adjust width
      if (maxWidth < 1e-5) {
        maxWidth = 1;
      }
      // const width = e.getAttribute("width")!;
      // const height = e.getAttribute("height")!;

      /// Prepare scale
      // scale derived from svg width and container with.
      const computedScale = container.width ? container.width / maxWidth : 1;
      // respect current scale ratio
      const scale = 1 / (this.currentScaleRatio * computedScale);
      const fontSize = 12 * scale;

      /// Calculate new width, height
      // 5pt height margin, 0pt width margin (it is buggy to add width margin)
      const heightMargin = this.isContentPreview ? 6 * scale : 5 * scale;
      const widthMargin = 0;
      const newWidth = maxWidth + 2 * widthMargin;

      /// Apply new pages
      let accumulatedHeight = 0;
      const firstPage = (nextPages.length ? nextPages[0] : undefined)!;
      let firstRect: SVGRectElement = undefined!;

      const pagesInCanvasMode: CanvasPage[] = [];
      /// Number to canvas page mapping
      const n2CMapping = new Map<number, CanvasPage>();
      const createCanvasPageOn = (nextPage: SvgPage) => {
        const { elem, width, height, index } = nextPage;
        const pg: CanvasPage = {
          tag: "canvas",
          index,
          width,
          height,
          container: undefined!,
          elem: undefined!,
          inserter: (pageInfo) => {
            const foreignObject = document.createElementNS(
              "http://www.w3.org/2000/svg",
              "foreignObject"
            );
            elem.appendChild(foreignObject);
            foreignObject.setAttribute("width", `${width}`);
            foreignObject.setAttribute("height", `${height}`);
            foreignObject.classList.add("typst-svg-mixin-canvas");
            foreignObject.prepend(pageInfo.container);
          },
        };
        n2CMapping.set(index, pg);
        pagesInCanvasMode.push(pg);
      };

      for (let i = 0; i < nextPages.length; i++) {
        /// Retrieve page width, height
        const nextPage = nextPages[i];
        const {
          width: pageWidth,
          height: pageHeight,
          elem: pageElem,
        } = nextPage;

        /// Switch a dummy svg page to canvas mode
        if (kShouldMixinCanvas && isDummyPatchElem(pageElem)) {
          /// Render this page as canvas
          createCanvasPageOn(nextPage);
          pageElem.setAttribute("data-mixin-canvas", "1");

          /// override reuse info for virtual DOM patching
          ///
          /// we cannot have much work to do, but we optimistically think of the canvas
          /// on the same page offset are the same canvas element.
          const offsetTag = `canvas:${nextPage.index}`;
          pageElem.setAttribute(TypstPatchAttrs.Tid, offsetTag);
          pageElem.setAttribute(TypstPatchAttrs.ReuseFrom, offsetTag);
        }

        /// center the page and add margin
        const calculatedPaddedX = (newWidth - pageWidth) / 2;
        const calculatedPaddedY =
          accumulatedHeight + (i == 0 ? 0 : heightMargin);
        const translateAttr = `translate(${calculatedPaddedX}, ${calculatedPaddedY})`;

        /// Create inner rectangle
        const innerRect = document.createElementNS(
          "http://www.w3.org/2000/svg",
          "rect"
        );
        innerRect.setAttribute("class", "typst-page-inner");
        innerRect.setAttribute("data-page-width", pageWidth.toString());
        innerRect.setAttribute("data-page-height", pageHeight.toString());
        innerRect.setAttribute(
          "width",
          Math.floor(pageWidth * INNER_RECT_UNIT).toString()
        );
        innerRect.setAttribute(
          "height",
          Math.floor(pageHeight * INNER_RECT_UNIT).toString()
        );
        innerRect.setAttribute("x", "0");
        innerRect.setAttribute("y", "0");
        innerRect.setAttribute(
          "transform",
          `${translateAttr} ${INNER_RECT_SCALE}`
        );
        if (this.pageColor) {
          innerRect.setAttribute("fill", this.pageColor);
        }
        // It is quite ugly
        // innerRect.setAttribute("stroke", "black");
        // innerRect.setAttribute("stroke-width", (2 * INNER_RECT_UNIT * scale).toString());
        // innerRect.setAttribute("stroke-opacity", "0.4");

        /// Move page to the correct position
        pageElem.setAttribute("transform", translateAttr);

        /// Insert rectangles
        // todo: this is buggy not preserving order?
        svg.insertBefore(innerRect, firstPage.elem);
        if (!firstRect) {
          firstRect = innerRect;
        }

        let pageHeightEnd =
          pageHeight + (i + 1 === nextPages.length ? 0 : heightMargin);

        if (this.isContentPreview) {
          // --typst-preview-toolbar-fg-color
          // create page number indicator
          // console.log('create page number indicator', scale);
          const pageNumberIndicator = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "text"
          );
          pageNumberIndicator.setAttribute(
            "class",
            "typst-preview-svg-page-number"
          );
          pageNumberIndicator.setAttribute("x", "0");
          pageNumberIndicator.setAttribute("y", "0");
          const pnPaddedX = calculatedPaddedX + pageWidth / 2;
          const pnPaddedY =
            calculatedPaddedY + pageHeight + heightMargin + fontSize / 2;
          pageNumberIndicator.setAttribute(
            "transform",
            `translate(${pnPaddedX}, ${pnPaddedY})`
          );
          pageNumberIndicator.setAttribute("font-size", fontSize.toString());
          pageNumberIndicator.textContent = `${i + 1}`;
          svg.append(pageNumberIndicator);

          pageHeightEnd += fontSize;
        } else {
          if (this.cursorPosition && this.cursorPosition[0] === i + 1) {
            const [_, x, y] = this.cursorPosition;
            const cursor = document.createElementNS(
              "http://www.w3.org/2000/svg",
              "circle"
            );
            cursor.setAttribute("cx", (x * INNER_RECT_UNIT).toString());
            cursor.setAttribute("cy", (y * INNER_RECT_UNIT).toString());
            cursor.setAttribute("r", (5 * scale * INNER_RECT_UNIT).toString());
            cursor.setAttribute("fill", "#86C166CC");
            cursor.setAttribute(
              "transform",
              `${translateAttr} ${INNER_RECT_SCALE}`
            );
            svg.appendChild(cursor);
          }
        }

        accumulatedHeight = calculatedPaddedY + pageHeightEnd;
      }

      /// Starts to stole and update canvas elements
      if (kShouldMixinCanvas) {
        /// Retrieves original pages
        for (const prev of this.hookedElem.firstElementChild?.children || []) {
          if (!prev.classList.contains("typst-page")) {
            continue;
          }
          // nextPage.elem.setAttribute('data-mixin-canvas', 'true');
          if (prev.getAttribute("data-mixin-canvas") !== "1") {
            continue;
          }

          const ch = prev.querySelector(".typst-svg-mixin-canvas");
          if (ch?.tagName === "foreignObject") {
            const canvasDiv = ch.firstElementChild as HTMLDivElement;

            const pageNumber = Number.parseInt(
              canvasDiv.getAttribute("data-page-number")!
            );
            const pageInfo = n2CMapping.get(pageNumber);
            if (pageInfo) {
              pageInfo.container = canvasDiv as HTMLDivElement;
              pageInfo.elem = canvasDiv.firstElementChild as HTMLDivElement;
            }
          }
        }

        this.createCanvas(pagesInCanvasMode);
        console.assert(
          this.canvasRenderCToken === undefined,
          "Noo!!: canvasRenderCToken should be undefined"
        );
        const tok = (this.canvasRenderCToken = new TypstCancellationToken());
        this.updateCanvas(pagesInCanvasMode, {
          cancel: tok,
        }).finally(() => {
          if (tok === this.canvasRenderCToken) {
            this.canvasRenderCToken = undefined;
          }
        });
      }

      if (this.isContentPreview) {
        accumulatedHeight += fontSize; // always add a bottom margin for last page number
      }

      /// Apply new width, height
      const newHeight = accumulatedHeight;

      /// Create outer rectangle
      if (firstPage) {
        const rectHeight = Math.ceil(newHeight).toString();

        const outerRect = document.createElementNS(
          "http://www.w3.org/2000/svg",
          "rect"
        );
        outerRect.setAttribute("class", "typst-page-outer");
        outerRect.setAttribute("data-page-width", newWidth.toString());
        outerRect.setAttribute("data-page-height", rectHeight);
        outerRect.setAttribute("width", newWidth.toString());
        outerRect.setAttribute("height", rectHeight);
        outerRect.setAttribute("x", "0");
        outerRect.setAttribute("y", "0");
        // white background
        outerRect.setAttribute("fill", this.backgroundColor);
        svg.insertBefore(outerRect, firstRect);
      }

      /// Update svg width, height information
      svg.setAttribute("viewBox", `0 0 ${newWidth} ${newHeight}`);
      svg.setAttribute("width", `${Math.ceil(newWidth)}`);
      svg.setAttribute("height", `${Math.ceil(newHeight)}`);
      svg.setAttribute("data-width", `${newWidth}`);
      svg.setAttribute("data-height", `${newHeight}`);

      /// Early rescale
      this.rescaleSvgOn(svg);
    }

    private get docWidth() {
      const svg = this.hookedElem.firstElementChild!;

      if (svg) {
        let svgWidth = Number.parseFloat(
          svg.getAttribute("data-width")! || svg.getAttribute("width")! || "1"
        );
        if (svgWidth < 1e-5) {
          svgWidth = 1;
        }
        return svgWidth;
      }

      return this.kModule.docWidth;
    }

    private statSvgFromDom() {
      const { width: containerWidth, boundingRect: containerBRect } =
        this.cachedDOMState;
      // scale derived from svg width and container with.
      // svg.setAttribute("data-width", `${newWidth}`);

      const computedRevScale = containerWidth
        ? this.docWidth / containerWidth
        : 1;
      // respect current scale ratio
      const revScale = computedRevScale / this.currentScaleRatio;
      const left = (window.screenLeft - containerBRect.left) * revScale;
      const top = (window.screenTop - containerBRect.top) * revScale;
      const width = window.innerWidth * revScale;
      const height = window.innerHeight * revScale;

      return { revScale, left, top, width, height };
    }
  };
}
