#import "../lib.typ": *

#set page(width: 500pt)

#assert.eq(mitex-convert("\alpha x"), "alpha x ")

Write inline equations like #mi("x") or #mi[y].

Also block equations:

#mitex(`
\partial_{u} \xi_{z}^{(1)} +{1\over u} \xi_{z}^{(1)}  = {1\over (\pi T R)^2 u}\left[ C_z H_{zz}'  + C_t H_{tz}' \right]\,.\label{gauge_eq_z_3_p}
`)
