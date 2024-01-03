// https://codesandbox.io/p/sandbox/codemirror-6-grammarly-latex-opwol7?file=%2Findex.js%3A1%2C1-27%2C2
import van, { State } from "vanjs-core";
const { div } = van.tags;

import {
  ChangeSet,
  EditorState,
  Range as EditorRange,
} from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import {
  syntaxHighlighting,
  defaultHighlightStyle,
  StreamLanguage,
} from "@codemirror/language";
import { stex } from "@codemirror/legacy-modes/mode/stex";
import {
  ViewPlugin,
  Decoration,
  DecorationSet,
  PluginValue,
} from "@codemirror/view";
import { syntaxTree } from "@codemirror/language";
import { EditorSDK, init } from "@grammarly/editor-sdk";
import { FsItemState } from "./underleaf-fs";

interface NativeSpellPlugin extends PluginValue {
  decorations: DecorationSet;
}

const nativeSpelling = () => {
  return ViewPlugin.define<NativeSpellPlugin>(
    () => {
      const buildDecorations = (view: EditorView) => {
        const decorations: EditorRange<Decoration>[] = [];

        const tree = syntaxTree(view.state);

        for (const { from, to } of view.visibleRanges) {
          tree.iterate({
            from,
            to,
            enter({ type, from, to }) {
              if (typesWithoutSpellcheck.includes(type.name)) {
                decorations.push(spellcheckDisabledMark.range(from, to));
              }
            },
          });
        }

        return Decoration.set(decorations);
      };

      const value: NativeSpellPlugin = {
        decorations: Decoration.none,
        update(update) {
          /// shouldRecalculate
          if (update.docChanged || update.viewportChanged) {
            value.decorations = buildDecorations(update.view);
          }
        },
      };
      return value;
    },
    {
      decorations: (value) => value.decorations,
    }
  );
};

const spellcheckDisabledMark = Decoration.mark({
  attributes: { spellcheck: "false" },
});

const typesWithoutSpellcheck = ["typeName", "atom"];

const extensions = [
  EditorView.contentAttributes.of({ spellcheck: "true" }),
  EditorView.lineWrapping,
  StreamLanguage.define(stex),
  syntaxHighlighting(defaultHighlightStyle),
  nativeSpelling(),
];

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8");
/// The editor component
export const Editor = (
  darkMode: State<boolean>,
  changeFocusFile: State<FsItemState | undefined>,
  focusFile: State<FsItemState | undefined>
) => {
  let updateListenerExtension = EditorView.updateListener.of(async (update) => {
    const f = focusFile.val;
    if (update.docChanged && f) {
      const c = encoder.encode(update.state.doc.toString());
      await window.$typst?.mapShadow(f.path, c);
      f.data.val = c;
      // console.log("update", f.path, decoder.decode(c));
    }
  });

  const state = EditorState.create({
    extensions: [
      ...extensions,
      EditorView.theme({
        "&": { height: "100%" },
      }),
      EditorView.darkTheme.of(darkMode.val),
      updateListenerExtension,
    ],
  });
  const view = new EditorView({ state });
  init("client_9m1fYK3MPQxwKsib5CxtpB").then((Grammarly: EditorSDK) => {
    Grammarly.addPlugin(view.contentDOM, {
      activation: "immediate",
    });
  });

  const vs = van.derive(() => {
    // console.log("focusFile.val", changeFocusFile.val);
    const path = changeFocusFile.val?.path;
    if (!changeFocusFile.val || !path) {
      return "";
    }
    if (path.endsWith(".png")) {
      return `Binary file ${path} is not shown`;
    }
    const data = changeFocusFile.val.data.val;
    return data ? decoder.decode(data) : "";
  });
  van.derive(() => {
    view.dispatch({
      changes: [
        ChangeSet.empty(0),
        {
          from: 0,
          insert: vs.val,
        },
      ],
    });
  });

  return div({ class: "mitex-input" }, view.dom);
};
