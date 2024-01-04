import { RenderSession } from "@myriaddreamin/typst.ts/dist/esm/renderer.mjs";
import { provideDebugJump } from "./traits/debug-jump.mjs";
import { provideSvg } from "./traits/svg.mjs";
import { provideCanvas } from "./traits/canvas.mjs";
import { TypstDocumentContext } from "./traits/base.mjs";
export interface ContainerDOMState {
  /// cached `hookedElem.offsetWidth` or `hookedElem.innerWidth`
  width: number;
  /// cached `hookedElem.offsetHeight` or `hookedElem.innerHeight`
  height: number;
  /// cached `hookedElem.getBoundingClientRect()`
  /// We only use `left` and `top` here.
  boundingRect: {
    left: number;
    top: number;
  };
}

export enum RenderMode {
  Svg,
  Canvas,
}

export enum PreviewMode {
  Doc,
  Slide,
}

interface Options {
  renderMode?: RenderMode;
  previewMode?: PreviewMode;
  isContentPreview?: boolean;
  sourceMapping?: boolean;
  retrieveDOMState?: () => ContainerDOMState;
}

class TypstDocumentImpl extends provideDebugJump(
  provideSvg(provideCanvas(TypstDocumentContext))
) {}

export class TypstDocument {
  private impl: TypstDocumentImpl;

  constructor(
    hookedElem: HTMLElement,
    public kModule: RenderSession,
    options?: Options
  ) {
    this.impl = new TypstDocumentImpl({ ...options, hookedElem, kModule });
    if (!this.impl.r) {
      throw new Error(`mode is not supported, ${options?.renderMode}`);
    }
  }

  dispose() {
    this.impl.dispose();
  }

  reset() {
    this.impl.reset();
  }

  addChangement(change: [string, string]) {
    this.impl.addChangement(change);
  }

  addViewportChange() {
    this.impl.addViewportChange();
  }

  setPageColor(color: string) {
    this.impl.pageColor = color;
    this.addViewportChange();
  }

  setPartialRendering(partialRendering: boolean) {
    this.impl.partialRendering = partialRendering;
  }

  setCursor(page: number, x: number, y: number) {
    this.impl.cursorPosition = [page, x, y];
  }

  setPartialPageNumber(page: number): boolean {
    if (page <= 0 || page > this.kModule.retrievePagesInfo().length) {
      return false;
    }
    this.impl.partialRenderPage = page - 1;
    this.addViewportChange();
    return true;
  }

  getPartialPageNumber(): number {
    return this.impl.partialRenderPage + 1;
  }

  setOutineData(outline: any) {
    this.impl.outline = outline;
    this.addViewportChange();
  }
}
