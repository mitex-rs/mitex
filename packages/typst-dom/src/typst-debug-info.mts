import { triggerRipple } from "./typst-animation.mjs";

const enum SourceMappingType {
  Text = 0,
  Group = 1,
  Image = 2,
  Shape = 3,
  Page = 4,
}

// one-of following classes must be present:
// - typst-page
// - typst-group
// - typst-text
// - typst-image
// - typst-shape
const CssClassToType = [
  ["typst-text", SourceMappingType.Text],
  ["typst-group", SourceMappingType.Group],
  ["typst-image", SourceMappingType.Image],
  ["typst-shape", SourceMappingType.Shape],
  ["typst-page", SourceMappingType.Page],
] as const;

function castToSourceMappingElement(
  elem: Element
): [SourceMappingType, Element] | undefined {
  if (elem.classList.length === 0) {
    return undefined;
  }
  for (const [cls, ty] of CssClassToType) {
    if (elem.classList.contains(cls)) {
      return [ty, elem];
    }
  }
  return undefined;
}

function castToNestSourceMappingElement(
  elem: Element
): [SourceMappingType, Element] | undefined {
  while (elem) {
    const result = castToSourceMappingElement(elem);
    if (result) {
      return result;
    }
    let chs = elem.children;
    if (chs.length !== 1) {
      return undefined;
    }
    elem = chs[0];
  }

  return undefined;
}

function castChildrenToSourceMappingElement(
  elem: Element
): [SourceMappingType, Element][] {
  return Array.from(elem.children)
    .map(castToNestSourceMappingElement)
    .filter((x) => x) as [SourceMappingType, Element][];
}

export function removeSourceMappingHandler(docRoot: HTMLElement) {
  const prevSourceMappingHandler = (docRoot as any).sourceMappingHandler;
  if (prevSourceMappingHandler) {
    docRoot.removeEventListener("click", prevSourceMappingHandler);
    delete (docRoot as any).sourceMappingHandler;
    // console.log("remove removeSourceMappingHandler");
  }
}

function findIndexOfChild(elem: Element, child: Element) {
  const children = castChildrenToSourceMappingElement(elem);
  console.log(elem, "::", children, "=>", child);
  return children.findIndex((x) => x[1] === child);
}

export function installEditorJumpToHandler(svgDoc: any, docRoot: HTMLElement) {
  void castChildrenToSourceMappingElement;

  const findSourceLocation = (elem: Element) => {
    const visitChain: [SourceMappingType, Element][] = [];
    while (elem) {
      let srcElem = castToSourceMappingElement(elem);
      if (srcElem) {
        visitChain.push(srcElem);
      }
      if (elem === docRoot) {
        break;
      }
      elem = elem.parentElement!;
    }

    if (visitChain.length === 0) {
      return undefined;
    }

    for (let idx = 1; idx < visitChain.length; idx++) {
      const childIdx = findIndexOfChild(
        visitChain[idx][1],
        visitChain[idx - 1][1]
      );
      if (childIdx < 0) {
        return undefined;
      }
      (visitChain[idx - 1][1] as any) = childIdx;
    }

    visitChain.reverse();

    const pg = visitChain[0];
    if (pg[0] !== SourceMappingType.Page) {
      return undefined;
    }
    const childIdx = findIndexOfChild(pg[1].parentElement!, visitChain[0][1]);
    if (childIdx < 0) {
      return undefined;
    }
    (visitChain[0][1] as any) = childIdx;

    const sourceNodePath = visitChain.flat();

    // The page always shadowed by group, so we remove it.
    // todo: where should I remove group under page? Putting here is a bit magical.
    sourceNodePath.splice(2, 2);
    console.log(sourceNodePath);

    return svgDoc.get_source_loc(sourceNodePath);
  };

  removeSourceMappingHandler(docRoot);
  const sourceMappingHandler = ((docRoot as any).sourceMappingHandler = (
    event: MouseEvent
  ) => {
    let elem = event.target! as Element;

    const sourceLoc = findSourceLocation(elem);
    if (!sourceLoc) {
      return;
    }
    console.log("source location", sourceLoc);

    const triggerWindow = document.body || document.firstElementChild;
    const basePos = triggerWindow.getBoundingClientRect();

    // const vw = window.innerWidth || 0;
    const left = event.clientX - basePos.left;
    const top = event.clientY - basePos.top;

    triggerRipple(
      triggerWindow,
      left,
      top,
      "typst-debug-react-ripple",
      "typst-debug-react-ripple-effect .4s linear"
    );

    window.typstWebsocket.send(`srclocation ${sourceLoc}`);
    return;
  });

  docRoot.addEventListener("click", sourceMappingHandler);
}
