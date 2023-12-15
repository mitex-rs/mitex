#let mitex-wasm = plugin("./mitex.wasm")

#let mitex-convert(s) = str(mitex-wasm.convert(bytes(s)))

#let mitex-scope = (
  frac: (num, den) => $(num)/(den)$,
)

#let mitex(it, block: true, mitex-scope: mitex-scope) = {
  let res = mitex-convert({
    if type(it) == str {
      it
    } else if type(it) == content {
      it.text
    } else {
      panic("Unsupported type: " + str(type(it)))
    }
  })
  if block {
    eval("$ " + res + " $", scope: mitex-scope)
  } else {
    eval("$" + res + "$", scope: mitex-scope)
  }
}

#let mi = mitex.with(block: false)
