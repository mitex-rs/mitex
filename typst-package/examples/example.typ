#import "../lib.typ": *

#set page(width: 500pt)

#assert.eq(mitex-convert("\alpha x"), "alpha x ")

test #mi("x") yd

#mitex(`
  \begin{aligned}
    f(x) & = \begin{bmatrix}
      1 & 2 & 3  \\
      1 & 2 & 3  \\
      1 & 2 & 3  \\
    \end{bmatrix} \\
    & = 2  \\
    & = \beta \hat{x} \leqslant y \\
  \end{aligned}
`)