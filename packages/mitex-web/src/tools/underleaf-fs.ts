import van, { State } from "vanjs-core";
const { div, span } = van.tags;

import LightningFS from "@isomorphic-git/lightning-fs";
import { GitHttpRequest } from "isomorphic-git/http/web";

const dirJoin = (...args: string[]) => args.join("/");

/// https://stackoverflow.com/questions/21797299/convert-base64-string-to-arraybuffer
const bufferToBase64Url = async (data: Uint8Array) => {
  // Use a FileReader to generate a base64 data URI
  const base64url = await new Promise<string | null>((r, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      if (typeof result === "string" || result === null) {
        r(result);
      }
      reject(new Error("Unexpected result type"));
    };
    reader.readAsDataURL(
      new Blob([data], { type: "application/octet-binary" })
    );
  });

  // remove the `data:...;base64,` part from the start
  return base64url;
};

async function base64UrlToBuffer(base64Url: string) {
  const res = await fetch(base64Url);
  const buffer = await res.arrayBuffer();
  return new Uint8Array(buffer);
}

export class FsItemState {
  constructor(
    public path: string,
    public data: State<Uint8Array>,
    public deleted: State<boolean>
  ) {}
  async serialize() {
    return {
      path: this.path,
      data: await bufferToBase64Url(this.data.val),
      deleted: this.deleted.val,
    };
  }

  clone() {
    return new FsItemState(this.path, van.state(this.data.val), this.deleted);
  }
}

const FsItem =
  (prefix: string, { path, deleted }: FsItemState) =>
  () => {
    return deleted.val
      ? null
      : div(
          span({
            class: "mitex-dir-view-icon",
            innerHTML: `<svg width="10px" height="10px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M10 17L8 15L10 13M14 13L16 15L14 17M13 3H8.2C7.0799 3 6.51984 3 6.09202 3.21799C5.71569 3.40973 5.40973 3.71569 5.21799 4.09202C5 4.51984 5 5.0799 5 6.2V17.8C5 18.9201 5 19.4802 5.21799 19.908C5.40973 20.2843 5.71569 20.5903 6.09202 20.782C6.51984 21 7.0799 21 8.2 21H15.8C16.9201 21 17.4802 21 17.908 20.782C18.2843 20.5903 18.5903 20.2843 18.782 19.908C19 19.4802 19 18.9201 19 17.8V9M13 3L19 9M13 3V8C13 8.55228 13.4477 9 14 9H19" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`,
            style: "margin-right: 0.5em;",
          }),
          () => span(path.replace(prefix, ""))
          // a({ onclick: () => (deleted.val = true) }, "❌")
        );
  };

class FsState {
  private constructor(
    public pathSet: Map<string, FsItemState>,
    public fsList: FsItemState[]
  ) {}

  async save() {
    console.log("save local edition state");
    localStorage.setItem(
      "fsState",
      JSON.stringify(await Promise.all(this.fsList.map((t) => t.serialize())))
    );
  }

  static readonly load = async () => {
    const x = new Map<string, FsItemState>();
    const y = (
      await Promise.all(
        JSON.parse(localStorage.getItem("fsState") ?? "[]").map(
          async (t: any) => {
            if (x.has(t.path)) {
              return;
            }
            const s = new FsItemState(
              t.path,
              van.state(await base64UrlToBuffer(t.data)),
              van.state(false)
            );
            x.set(t.path, s);
            return s;
          }
        )
      )
    ).filter((t) => t);
    return new FsState(x, y);
  };

  add(path: string, data: Uint8Array) {
    const prev = this.pathSet.get(path);
    if (prev) {
      prev.data.val = data;
      return this;
    }
    const state = new FsItemState(path, van.state(data), van.state(false));
    this.fsList.push(state);
    this.fsList.sort((a, b) => a.path.localeCompare(b.path));
    this.pathSet.set(path, state);
    return new FsState(this.pathSet, this.fsList);
  }
}

export interface DirectoryViewState {
  compilerLoaded: State<boolean>;
  changeFocusFile: State<FsItemState | undefined>;
  focusFile: State<FsItemState | undefined>;
  reloadBell: State<boolean>;
}

/// The directory component
export const DirectoryView = ({
  compilerLoaded,
  changeFocusFile,
  focusFile,
  reloadBell,
}: DirectoryViewState) => {
  /// Capture compiler load status
  const remoteGitFsLoaded = van.state(false);
  const localStorageLoaded = van.state(false);
  const loaded = van.state(false);
  const fsState = van.state<FsState | undefined>(undefined);

  /// Internal fields
  const fs = new LightningFS("fs", { wipe: true, db: undefined! });
  const gitRepoDir = "/repo";
  const projectDir = "/repo/fixtures/underleaf/ieee";

  /// Store localstorage data whenver fs state changes
  van.derive(() => fsState.val?.save());

  /// Reload all files to compiler and application
  const reloadAll = async (state: FsState) => {
    const $typst = window.$typst;
    await Promise.all(
      (fsState.val?.fsList || []).map(async (f) => {
        return await $typst.mapShadow(f.path, f.data.val);
      })
    );

    focusFile.val = state.fsList.find(
      (t) => t.path === "/repo/fixtures/underleaf/ieee/main.tex"
    );
    changeFocusFile.val = focusFile.val?.clone();
  };

  /// Task: load localstorage data to fs state
  (async () => {
    fsState.val = await FsState.load();
  })();
  van.derive(async () => {
    if (
      loaded.val ||
      localStorageLoaded.val ||
      !(compilerLoaded.val && fsState.val)
    ) {
      return;
    }
    localStorageLoaded.val = true;

    reloadAll(fsState.val);
    console.log("localstorage fs done");
    reloadBell.val = true;
  });

  /// Task: load remote data to fs
  (async () => {
    const git = await import("isomorphic-git/index.umd.min");
    const http = await import("isomorphic-git/http/web");
    const g = {
      fs,
      http: {
        request(h: GitHttpRequest) {
          h.url = `https://underleaf.mgt.workers.dev/?${h.url}`;
          return http.request(h);
        },
      },
      dir: gitRepoDir,
      url: "https://github.com/mitex-rs/underleaf",
      ref: "main",
    };
    try {
      await fs.promises.readFile("/repo/.git/config");
      await git.fetch(g);
    } catch (e) {
      await git.clone(g);
    }
    await git.setConfig({ ...g, path: "user.name", value: "mitex" });
    await git.setConfig({ ...g, path: "user.email", value: "me@mitex.com" });
    await git.checkout({ ...g, force: true });
    remoteGitFsLoaded.val = true;
  })();
  van.derive(async () => {
    if (
      loaded.val ||
      !(compilerLoaded.val && remoteGitFsLoaded.val && fsState.val)
    ) {
      return;
    }
    loaded.val = true;

    console.log("start to load fs to compiler");

    for (const fileName of await fs.promises.readdir(projectDir)) {
      const filePath = dirJoin(projectDir, fileName);
      const data = (await fs.promises.readFile(filePath)) as Uint8Array;
      // const decoder = new TextDecoder("utf-8");
      // console.log(filePath, decoder.decode(data));
      fsState.val = fsState.val.add(filePath, data);
    }

    reloadAll(fsState.val);
    console.log("git fs done");
    reloadBell.val = true;
  });

  return div(
    {
      class: "mitex-dir-view",
    },
    (_dom?: Element) =>
      div(
        fsState.val?.fsList
          .filter((t) => !t.path.endsWith(".typ"))
          .map((t) => FsItem(projectDir + "/", t)) || []
      )
  );
};
