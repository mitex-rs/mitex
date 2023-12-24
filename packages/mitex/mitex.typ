#import "specs/mod.typ": mitex-scope
#import "@preview/xarrow:0.2.0": xarrow
#let mitex-wasm = plugin("./mitex.wasm")

#let mitex-convert(it, spec: bytes(())) = {
  str(mitex-wasm.convert_math(bytes({
    if type(it) == str {
      it
    } else if type(it) == content and it.has("text") {
      it.text
    } else {
      panic("Unsupported type: " + str(type(it)))
    }
  }), spec))
}

#let mitex(it, block: true, numbering: auto, supplement: auto) = {
  let res = mitex-convert(it)
  let eval-res = eval("$" + res + "$", scope: mitex-scope)
  if numbering == auto and supplement == auto {
    math.equation(block: block, eval-res)
  } else if numbering == auto {
    math.equation(block: block, eval-res, supplement: supplement)
  } else if supplement == auto {
    math.equation(block: block, eval-res, numbering: numbering)
  } else {
    math.equation(block: block, eval-res, numbering: numbering, supplement: supplement)
  }
}

#let mi = mitex.with(block: false)
