import type { RenderSession } from "@myriaddreamin/typst.ts/dist/esm/renderer.mjs";
import { ContainerDOMState } from "../typst-doc.mjs";


export type GConstructor<T = {}> = new (...args: any[]) => T;

interface TypstDocumentFacade {
    rescale(): void;
}

class TypstDocumentContext {
    public hookedElem: HTMLElement;
    public kModule: RenderSession;
    public opts: any;
    public modes: [string, TypstDocumentFacade][] = [];


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

    constructor(opts: { hookedElem: HTMLElement, kModule: RenderSession }) {
        this.hookedElem = opts.hookedElem;
        this.kModule = opts.kModule;
        this.opts = opts;
    }

    static derive(ctx: any, mode: string) {
        return ['rescale'].reduce((acc: any, x: string) => { acc[x] = ctx[`${x}$${mode}`]; return acc; }, {} as TypstDocumentFacade);
    }

    registerMode(mode: any) {
        this.modes.push([mode, TypstDocumentContext.derive(this, mode)]);
    }
}

interface TypstCanvasDocument {
    renderCanvas(): any;
}

function provideCanvas<TBase extends GConstructor<TypstDocumentContext>>(Base: TBase)
    : TBase & GConstructor<TypstCanvasDocument> {
    return class extends Base {

        constructor(...args: any[]) {
            super(...args);
            this.registerMode("canvas");
        }

        renderCanvas() {
        }

        rescale$canvas() {
            // get dom state from cache, so we are free from layout reflowing
            // Note: one should retrieve dom state before rescale
            // const { width: cwRaw, height: ch } = this.cachedDOMState;
            // const cw = (this.isContentPreview ? (cwRaw - 10) : cwRaw);

            // // get dom state from cache, so we are free from layout reflowing
            // const docDiv = this.hookedElem.firstElementChild! as HTMLDivElement;
            // if (!docDiv) {
            //     return;
            // }

            // let isFirst = true;

            // const rescale = (canvasContainer: HTMLElement) => {
            //     // console.log(ch);
            //     // if (isFirst) {
            //     //     isFirst = false;
            //     //     canvasContainer.style.marginTop = `0px`;
            //     // } else {
            //     //     canvasContainer.style.marginTop = `${this.isContentPreview ? 6 : 5}px`;
            //     // }
            //     let elem = canvasContainer.firstElementChild as HTMLDivElement;

            //     const canvasWidth = Number.parseFloat(elem.getAttribute("data-page-width")!);
            //     const canvasHeight = Number.parseFloat(elem.getAttribute("data-page-height")!);

            //     this.currentRealScale =
            //         this.previewMode === PreviewMode.Slide ?
            //             Math.min(cw / canvasWidth, ch / canvasHeight) :
            //             cw / canvasWidth;
            //     const scale = this.currentRealScale * this.currentScaleRatio;

            //     // apply scale
            //     const appliedScale = (scale / this.pixelPerPt).toString();


            //     // set data applied width and height to memoize change
            //     if (elem.getAttribute("data-applied-scale") !== appliedScale) {
            //         elem.setAttribute("data-applied-scale", appliedScale);
            //         // apply translate
            //         const scaledWidth = Math.ceil(canvasWidth * scale);
            //         const scaledHeight = Math.ceil(canvasHeight * scale);

            //         elem.style.width = `${scaledWidth}px`;
            //         elem.style.height = `${scaledHeight}px`;
            //         elem.style.transform = `scale(${appliedScale})`;

            //         if (this.previewMode === PreviewMode.Slide) {

            //             const widthAdjust = Math.max((cw - scaledWidth) / 2, 0);
            //             const heightAdjust = Math.max((ch - scaledHeight) / 2, 0);
            //             docDiv.style.transform = `translate(${widthAdjust}px, ${heightAdjust}px)`;
            //         }
            //     }
            // }

            // if (this.isContentPreview) {
            //     isFirst = false;
            //     const rescaleChildren = (elem: HTMLElement) => {
            //         for (const ch of elem.children) {
            //             let canvasContainer = ch as HTMLElement;
            //             if (canvasContainer.classList.contains('typst-page')) {
            //                 rescale(canvasContainer);
            //             }
            //             if (canvasContainer.classList.contains('typst-outline')) {
            //                 rescaleChildren(canvasContainer);
            //             }
            //         }
            //     }

            //     rescaleChildren(docDiv);
            // } else {
            //     for (const ch of docDiv.children) {
            //         let canvasContainer = ch as HTMLDivElement;
            //         if (!canvasContainer.classList.contains('typst-page')) {
            //             continue;
            //         }
            //         rescale(canvasContainer);
            //     }
            // }
        }

    }
}

export const traits = {
    TypstDocumentContext,
    canvas: provideCanvas,
}

