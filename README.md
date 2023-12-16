# [Mitex](https://github.com/OrangeX4/typst-mitex)

Convert LaTeX equation to Typst equation, powered by wasm.

This is not actually a parser; rather, it is a **tokenizer** based on regular expressions. After tokenization is complete, a **direct translation** is performed, such as `{` to `(` and `}` to `)`.

Therefore, it is **FAST**, while the disadvantage is that it cannot guarantee complete compatibility with LaTeX equations.


## Limitations

- It will directly translate all occurrences of `}{` into `,` to support `\frac{a}{b}` to `frac(a, b)`.
- Syntax like `\hat x` is not allowed; you must use `\hat{x}` instead.


## Example

```typst
#import "./typst-package/lib.typ": *
// #import "@preview/mitex:0.1.0": *

#assert.eq(mitex-convert("\alpha x"), "alpha x ")

Write inline equations like #mi("x") or #mi[y].

Also block equations:

#mitex(`
  \int_1^2 x \mathrm{d} x
`)
```

![example](typst-package/examples/example.png)
