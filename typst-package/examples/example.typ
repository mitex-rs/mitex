#import "../lib.typ": *

#set page(width: 500pt)

#assert.eq(mitex-convert("\alpha x"), "alpha x ")

Write inline equations like #mi("x") or #mi[y].

Also block equations:

#mitex(`
  \int_1^2 x \mathrm{d} x
`)
