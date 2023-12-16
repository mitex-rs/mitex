#let mitex-wasm = plugin("./mitex.wasm")

#let mitex-convert(it) = {
  str(mitex-wasm.convert(bytes({
    if type(it) == str {
      it
    } else if type(it) == content and it.has("text") {
      it.text
    } else {
      panic("Unsupported type: " + str(type(it)))
    }
  })))
}

#let mitex-scope = (
  frac: (num, den) => $(num)/(den)$,
  cfrac: (num, den) => $display((num)/(den))$,
  dfrac: (num, den) => $display((num)/(den))$,
  tfrac: (num, den) => $inline((num)/(den))$,
)

#let mitex(it, block: true, mitex-scope: mitex-scope) = {
  let res = mitex-convert(it)
  if block {
    eval("$ " + res + " $", scope: mitex-scope)
  } else {
    eval("$" + res + "$", scope: mitex-scope)
  }
}

#let mi = mitex.with(block: false)
