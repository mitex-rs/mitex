import {
  installEditorJumpToHandler,
  removeSourceMappingHandler,
} from "../typst-debug-info.mjs";
import { GConstructor, TypstDocumentContext } from "./base.mjs";

export interface TypstDebugJumpDocument {}

export function provideDebugJump<
  TBase extends GConstructor<TypstDocumentContext>
>(Base: TBase): TBase & GConstructor<TypstDebugJumpDocument> {
  return class DebugJumpDocument extends Base {
    constructor(...args: any[]) {
      super(...args);
      if (this.opts.sourceMapping !== false) {
        installEditorJumpToHandler(this.kModule, this.hookedElem);
        this.disposeList.push(() => {
          if (this.hookedElem) {
            removeSourceMappingHandler(this.hookedElem);
          }
        });
      }
    }
  };
}
