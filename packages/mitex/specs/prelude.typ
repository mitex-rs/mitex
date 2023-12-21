
#let define-sym(s, sym: none) = {
  ((kind: "alias-sym", alias: s), if sym != none { (alias: s, handle: sym) } else { none })
}

#let define-greedy-cmd(s, handle: none) ={
  ((kind: "greedy-cmd", alias: s), if handle != none { (alias: s, handle: handle) } else { none })
}

#let define-infix-cmd(s, handle: none) ={
  ((kind: "infix-cmd", alias: s), if handle != none { (alias: s, handle: handle) } else { none })
}

#let define-glob-cmd(pat, s, handle: none) ={
  ((kind: "glob-cmd", pattern: pat, alias: s), if handle != none { (alias: s, handle: handle) } else { none })
}

#let define-cmd(num, alias: none, handle: none) = {
  ((
    kind: "cmd",
    args: ( "kind": "right", "pattern": ( kind: "fixed-len", len: num ) ),
    alias: alias,
  ), if handle != none { (alias: alias, handle: handle) } else { none })
}

#let define-env(num, alias: none, handle: none) = {
  ((
    kind: "env",
    args: if num != none {
      ( kind: "fixed-len", len: num )
    } else {
      ( kind: "none" )
    },
    ctx_feature: ( kind: "none" ),
    alias: alias,
  ), if handle != none { (alias: s, handle: handle) } else { none })
}

#let define-cases-env(alias: none, handle: none) = {
  ((
    kind: "env",
    args: ( kind: "none" ),
    ctx_feature: ( kind: "is-cases" ),
    alias: alias,
  ), if handle != none { (alias: alias, handle: handle) } else { none })
}

#let define-matrix-env(num, alias: none, handle: none) = {
  ((
    kind: "env",
    args: if num != none {
      ( kind: "fixed-len", len: num )
    } else {
      ( kind: "none" )
    },
    ctx_feature: ( kind: "is-matrix" ),
    alias: alias,
  ), if handle != none { (alias: alias, handle: handle) } else { none })
}

#let sym = ((kind: "sym"), none)
#let of-sym(handle) = ((kind: "sym"), (handle: handle))
#let left1-op(alias) = ((kind: "cmd", args: ( kind: "left1" ), alias: alias), none)
#let cmd1 = ((kind: "cmd1"), none)
#let cmd2 = ((kind: "cmd2"), none)
#let matrix-env = ((kind: "matrix-env"), none)
#let normal-env(handle) = ((kind: "normal-env"), (handle: handle))

#let process-spec(definitions) = {
  let spec = (:)
  let scope = (:)
  for (key, value) in definitions.pairs() {
    spec.insert(key, value.at(0))
    if value.at(1) != none {
      if "alias" in value.at(1) and type(value.at(1).alias) == str {
        scope.insert(value.at(1).alias, value.at(1).handle)
      } else {
        scope.insert(key, value.at(1).handle)
      }
    }
  }
  (spec: spec, scope: scope)
}
