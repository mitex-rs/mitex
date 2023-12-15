#import "../lib.typ": *

#set page(width: 500pt)

#assert.eq(mitex-convert("abc"), "a b c ")

test #mi("x") yd

#mitex(`
f(x) & = \begin{bmatrix}
  1 & 2 & 3  \\
  1 & 2 & 3  \\
  1 & 2 & 3  \\
\end{bmatrix} \\
& =2  \ 
& = del  \\
`)
