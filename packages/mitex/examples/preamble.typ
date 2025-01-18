#import "../lib.typ": *

#let preamble = ```tex
\newcommand{\f}[2]{#1f(#2)}
```

#let mitex = (it, ..args) => mitex.with(..args)({
  (preamble, it).map(it => it.text).join("\n")
})

#mitex(```latex
\f\relax{x} = \int_{-\infty}^\infty
  \f\hat\xi\,e^{2 \pi i \xi x}
  \,d\xi
```)
