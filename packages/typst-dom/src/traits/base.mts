import type { RenderSession } from "@myriaddreamin/typst.ts/dist/esm/renderer.mjs";
import { ContainerDOMState, PreviewMode, RenderMode } from "../typst-doc.mjs";
import { TypstCancellationToken } from "./cancel.mjs";
import {
  installEditorJumpToHandler,
  removeSourceMappingHandler,
} from "../typst-debug-info.mjs";

export type GConstructor<T = {}> = new (...args: any[]) => T;

interface TypstDocumentFacade {
  mode: RenderMode;
  rescale(): void;
  rerender(): Promise<void>;
  postRender(): void;
}

interface Options {
  hookedElem: HTMLElement;
  kModule: RenderSession;
  renderMode?: RenderMode;
  previewMode?: PreviewMode;
  isContentPreview?: boolean;
  sourceMapping?: boolean;
  retrieveDOMState?: () => ContainerDOMState;
}

export class TypstDocumentContext {
  public hookedElem: HTMLElement;
  public kModule: RenderSession;
  public opts: any;
  public modes: [string, TypstDocumentFacade][] = [];

  /// Configuration fields

  /// enable partial rendering
  partialRendering: boolean = true;
  /// underlying renderer
  r: TypstDocumentFacade;
  /// preview mode
  previewMode: PreviewMode = PreviewMode.Doc;
  /// whether this is a content preview
  isContentPreview: boolean = false;
  /// whether this content preview will mix outline titles
  isMixinOutline: boolean = false;
  /// background color
  backgroundColor: string = "black";
  /// default page color (empty string means transparent)
  pageColor: string = "white";
  /// pixel per pt
  pixelPerPt: number = 3;
  /// customized way to retrieving dom state
  retrieveDOMState: () => ContainerDOMState;

  /// State fields

  /// whether svg is updating (in triggerSvgUpdate)
  isRendering: boolean = false;
  /// whether kModule is initialized
  moduleInitialized: boolean = false;
  /// patch queue for updating data.
  patchQueue: [string, string][] = [];
  /// resources to dispose
  private disposeList: (() => void)[] = [];
  /// canvas render ctoken
  canvasRenderCToken?: TypstCancellationToken;

  /// There are two scales in this class: The real scale is to adjust the size
  /// of `hookedElem` to fit the svg. The virtual scale (scale ratio) is to let
  /// user zoom in/out the svg. For example:
  /// + the default value of virtual scale is 1, which means the svg is totally
  ///   fit in `hookedElem`.
  /// + if user set virtual scale to 0.5, then the svg will be zoomed out to fit
  ///   in half width of `hookedElem`. "real" current scale of `hookedElem`
  currentRealScale: number = 1;
  /// "virtual" current scale of `hookedElem`
  currentScaleRatio: number = 1;
  /// timeout for delayed viewport change
  vpTimeout: any = undefined;
  /// sampled by last render time.
  sampledRenderTime: number = 0;
  /// page to partial render
  partialRenderPage: number = 0;
  /// outline data
  outline: any = undefined;
  /// cursor position in form of [page, x, y]
  cursorPosition?: [number, number, number] = undefined;
  // id: number = rnd++;

  /// Cache fields

  /// cached state of container, default to retrieve state from `this.hookedElem`
  cachedDOMState: ContainerDOMState = {
    width: 0,
    height: 0,
    boundingRect: {
      left: 0,
      top: 0,
    },
  };

  constructor(opts: Options) {
    this.hookedElem = opts.hookedElem;
    this.kModule = opts.kModule;
    this.opts = opts;

    // todo
    // if (this.isContentPreview) {
    //   // content preview has very bad performance without partial rendering
    //   this.partialRendering = true;
    //   this.renderMode = RenderMode.Canvas;
    //   this.pixelPerPt = 1;
    //   this.isMixinOutline = true;
    // }

    /// Apply configuration
    {
      const { renderMode, previewMode, isContentPreview, retrieveDOMState } =
        opts || {};
      this.partialRendering = false;
      const modeStr = RenderMode[renderMode || RenderMode.Svg].toLowerCase();
      this.r = this.modes.find((x) => x[0] === modeStr)?.[1]!;
      if (!this.r) {
        throw new Error(`unknown render mode ${modeStr}`);
      }

      if (previewMode !== undefined) {
        this.previewMode = previewMode;
      }
      this.isContentPreview = isContentPreview || false;
      this.retrieveDOMState =
        retrieveDOMState ||
        (() => {
          return {
            width: this.hookedElem.offsetWidth,
            height: this.hookedElem.offsetHeight,
            boundingRect: this.hookedElem.getBoundingClientRect(),
          };
        });
      this.backgroundColor = getComputedStyle(
        document.documentElement
      ).getPropertyValue("--typst-preview-background-color");
    }

    // if init scale == 1
    // hide scrollbar if scale == 1

    this.hookedElem.classList.add("hide-scrollbar-x");
    this.hookedElem.parentElement?.classList.add("hide-scrollbar-x");
    if (this.previewMode === PreviewMode.Slide) {
      this.hookedElem.classList.add("hide-scrollbar-y");
      this.hookedElem.parentElement?.classList.add("hide-scrollbar-y");
    }

    if (this.r.mode === RenderMode.Svg && opts?.sourceMapping !== false) {
      installEditorJumpToHandler(this.kModule, this.hookedElem);
      this.disposeList.push(() => {
        if (this.hookedElem) {
          removeSourceMappingHandler(this.hookedElem);
        }
      });
    }
    this.installCtrlWheelHandler();
  }

  reset() {
    this.kModule.reset();
    this.moduleInitialized = false;
  }

  dispose() {
    const disposeList = this.disposeList;
    this.disposeList = [];
    disposeList.forEach((x) => x());
  }

  static derive(ctx: any, mode: string) {
    return ["rescale", "rerender", "postRender"].reduce(
      (acc: any, x: string) => {
        acc[x] = ctx[`${x}$${mode}`];
        console.assert(acc[x] !== undefined, `${x}$${mode} is undefined`);
        return acc;
      },
      {} as TypstDocumentFacade
    );
  }

  registerMode(mode: any) {
    this.modes.push([mode, TypstDocumentContext.derive(this, mode)]);
  }

  private installCtrlWheelHandler() {
    // Ctrl+scroll rescaling
    // will disable auto resizing
    // fixed factors, same as pdf.js
    const factors = [
      0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1, 1.1, 1.3, 1.5, 1.7, 1.9,
      2.1, 2.4, 2.7, 3, 3.3, 3.7, 4.1, 4.6, 5.1, 5.7, 6.3, 7, 7.7, 8.5, 9.4, 10,
    ];
    const wheelEventHandler = (event: WheelEvent) => {
      if (event.ctrlKey) {
        event.preventDefault();

        // retrieve dom state before any operation
        this.cachedDOMState = this.retrieveDOMState();

        if (window.onresize !== null) {
          // is auto resizing
          window.onresize = null;
        }
        const prevScaleRatio = this.currentScaleRatio;
        // Get wheel scroll direction and calculate new scale
        if (event.deltaY < 0) {
          // enlarge
          if (this.currentScaleRatio >= factors.at(-1)!) {
            // already large than max factor
            return;
          } else {
            this.currentScaleRatio = factors
              .filter((x) => x > this.currentScaleRatio)
              .at(0)!;
          }
        } else if (event.deltaY > 0) {
          // reduce
          if (this.currentScaleRatio <= factors.at(0)!) {
            return;
          } else {
            this.currentScaleRatio = factors
              .filter((x) => x < this.currentScaleRatio)
              .at(-1)!;
          }
        } else {
          // no y-axis scroll
          return;
        }
        const scrollFactor = this.currentScaleRatio / prevScaleRatio;
        const scrollX = event.pageX * (scrollFactor - 1);
        const scrollY = event.pageY * (scrollFactor - 1);

        // hide scrollbar if scale == 1
        if (Math.abs(this.currentScaleRatio - 1) < 1e-5) {
          this.hookedElem.classList.add("hide-scrollbar-x");
          this.hookedElem.parentElement?.classList.add("hide-scrollbar-x");
          if (this.previewMode === PreviewMode.Slide) {
            this.hookedElem.classList.add("hide-scrollbar-y");
            this.hookedElem.parentElement?.classList.add("hide-scrollbar-y");
          }
        } else {
          this.hookedElem.classList.remove("hide-scrollbar-x");
          this.hookedElem.parentElement?.classList.remove("hide-scrollbar-x");
          if (this.previewMode === PreviewMode.Slide) {
            this.hookedElem.classList.remove("hide-scrollbar-y");
            this.hookedElem.parentElement?.classList.remove("hide-scrollbar-y");
          }
        }

        // reserve space to scroll down
        const svg = this.hookedElem.firstElementChild! as SVGElement;
        if (svg) {
          const scaleRatio = this.getSvgScaleRatio();

          const dataHeight = Number.parseFloat(
            svg.getAttribute("data-height")!
          );
          const scaledHeight = Math.ceil(dataHeight * scaleRatio);

          // we increase the height by 2 times.
          // The `2` is only a magic number that is large enough.
          this.hookedElem.style.height = `${scaledHeight * 2}px`;
        }

        // make sure the cursor is still on the same position
        window.scrollBy(scrollX, scrollY);
        // toggle scale change event
        this.addViewportChange();

        return false;
      }
    };

    const vscodeAPI = typeof acquireVsCodeApi !== "undefined";
    if (vscodeAPI) {
      window.addEventListener("wheel", wheelEventHandler, {
        passive: false,
      });
      this.disposeList.push(() => {
        window.removeEventListener("wheel", wheelEventHandler);
      });
    } else {
      document.body.addEventListener("wheel", wheelEventHandler, {
        passive: false,
      });
      this.disposeList.push(() => {
        document.body.removeEventListener("wheel", wheelEventHandler);
      });
    }
  }

  /// Get current scale from html to svg
  // Note: one should retrieve dom state before rescale
  getSvgScaleRatio() {
    const svg = this.hookedElem.firstElementChild as SVGElement;
    if (!svg) {
      return 0;
    }

    const container = this.cachedDOMState;

    const svgWidth = Number.parseFloat(
      svg.getAttribute("data-width") || svg.getAttribute("width") || "1"
    );
    const svgHeight = Number.parseFloat(
      svg.getAttribute("data-height") || svg.getAttribute("height") || "1"
    );
    this.currentRealScale =
      this.previewMode === PreviewMode.Slide
        ? Math.min(container.width / svgWidth, container.height / svgHeight)
        : container.width / svgWidth;

    return this.currentRealScale * this.currentScaleRatio;
  }

  private async processQueue(svgUpdateEvent: [string, string]) {
    const ctoken = this.canvasRenderCToken;
    if (ctoken) {
      await ctoken.cancel();
      await ctoken.wait();
      this.canvasRenderCToken = undefined;
      console.log("cancel canvas rendering");
    }
    let t0 = performance.now();
    let t1 = undefined;
    let t2 = undefined;
    const eventName = svgUpdateEvent[0];
    switch (eventName) {
      case "new":
      case "diff-v1": {
        if (eventName === "new") {
          this.reset();
        }
        this.kModule.manipulateData({
          action: "merge",
          data: svgUpdateEvent[1] as unknown as Uint8Array,
        });

        // todo: trigger viewport change once
        t1 = performance.now();
        await this.r.rerender();
        t2 = performance.now();
        this.moduleInitialized = true;
        break;
      }
      case "viewport-change": {
        if (!this.moduleInitialized) {
          console.log("viewport-change before initialization");
          t0 = t1 = t2 = performance.now();
          break;
        }
        t1 = performance.now();
        await this.r.rerender();
        t2 = performance.now();
        break;
      }
      default:
        console.log("svgUpdateEvent", svgUpdateEvent);
        t0 = t1 = t2 = performance.now();
        break;
    }

    /// perf event
    const d = (e: string, x: number, y: number) =>
      `${e} ${(y - x).toFixed(2)} ms`;
    this.sampledRenderTime = t2 - t0;
    console.log(
      [d("parse", t0, t1), d("rerender", t1, t2), d("total", t0, t2)].join(", ")
    );
  }

  private triggerUpdate() {
    if (this.isRendering) {
      return;
    }

    this.isRendering = true;
    const doUpdate = async () => {
      this.cachedDOMState = this.retrieveDOMState();

      if (this.patchQueue.length === 0) {
        this.isRendering = false;
        this.postprocessChanges();
        return;
      }
      try {
        // console.log('patchQueue', JSON.stringify(this.patchQueue.map(x => x[0])));
        while (this.patchQueue.length > 0) {
          await this.processQueue(this.patchQueue.shift()!);
          this.r.rescale();
        }

        requestAnimationFrame(doUpdate);
      } catch (e) {
        console.error(e);
        this.isRendering = false;
        this.postprocessChanges();
      }
    };
    requestAnimationFrame(doUpdate);
  }

  private postprocessChanges() {
    // case RenderMode.Svg: {
    // const docRoot = this.hookedElem.firstElementChild as SVGElement;
    // if (docRoot) {
    //   window.initTypstSvg(docRoot);
    //   this.rescale();
    // }

    this.r.postRender();

    // todo: abstract this
    if (this.previewMode === PreviewMode.Slide) {
      document.querySelectorAll(".typst-page-number-indicator").forEach((x) => {
        x.textContent = `${this.kModule.retrievePagesInfo().length}`;
      });
    }
  }

  addChangement(change: [string, string]) {
    if (change[0] === "new") {
      this.patchQueue.splice(0, this.patchQueue.length);
    }

    const pushChange = () => {
      this.vpTimeout = undefined;
      this.patchQueue.push(change);
      this.triggerUpdate();
    };

    if (this.vpTimeout !== undefined) {
      clearTimeout(this.vpTimeout);
    }

    if (change[0] === "viewport-change" && this.isRendering) {
      // delay viewport change a bit
      this.vpTimeout = setTimeout(pushChange, this.sampledRenderTime || 100);
    } else {
      pushChange();
    }
  }

  addViewportChange() {
    this.addChangement(["viewport-change", ""]);
  }
}
