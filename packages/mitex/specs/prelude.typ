
#let define-sym(s, sym: none) = {
  (kind: "alias-sym", alias: s)
}

#let define-greedy-cmd(s, handle: none) ={
  (kind: "greedy-cmd", alias: s)
}

#let define-infix-cmd(s, handle: none) ={
  (kind: "infix-cmd", alias: s)
}

#let define-glob-cmd(pat, s, handle: none) ={
  (kind: "glob-cmd", pattern: pat, alias: s)
}

#let define-cmd(num, alias: none, handle: none) = {
  (
    kind: "cmd",
    args: ( "kind": "right", "pattern": ( kind: "fixed-len", len: num ) ),
    alias: alias,
  )
}

#let define-env(num, alias: none, handle: none) = {
  (
    kind: "env",
    args: if num != none {
      ( kind: "fixed-len", len: num )
    } else {
      ( kind: "none" )
    },
    ctx_feature: ( kind: "none" ),
    alias: alias,
  )
}

#let define-matrix-env(num, alias: none, handle: none) = {
  (
    kind: "env",
    args: if num != none {
      ( kind: "fixed-len", len: num )
    } else {
      ( kind: "none" )
    },
    ctx_feature: ( kind: "is-matrix" ),
    alias: alias,
  )
}

#let sym = (kind: "sym")
#let of-sym(handle) = (kind: "sym")
#let cmd1 = (kind: "cmd1")
#let cmd2 = (kind: "cmd2")
#let left1-op = (kind: "left1-cmd")
#let matrix-env = (kind: "matrix-env")
#let normal-env = (kind: "normal-env")

#let _package-state = state("mitex-packages", ())
#let define-package(pkg) = _package-state.update(it => {
  // todo: verify type
  it.push(pkg)
  it
})
#let packages-at(loc) = _package-state.at(loc)
#let packages-all(loc) = _package-state.final(loc)
