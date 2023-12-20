#import "../lib.typ": *

#set page(width: 500pt)

#assert.eq(mitex-convert("\alpha x"), "alpha  x ")

Write inline equations like #mi("x") or #mi[y].

Also block equations:

#mitex("\alpha x" * 8000)

/*

last^1
17000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     647.5 ms ±  16.3 ms    [User: 109.4 ms, System: 23.4 ms]
  Range (min … max):   630.9 ms … 676.8 ms    10 runs
8000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     537.5 ms ±  23.3 ms    [User: 76.6 ms, System: 17.2 ms]
  Range (min … max):   509.1 ms … 581.7 ms    10 runs

last^2
17000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     758.4 ms ±  26.8 ms    [User: 185.3 ms, System: 31.8 ms]
  Range (min … max):   722.5 ms … 802.6 ms    10 runs
8000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     605.6 ms ±  18.1 ms    [User: 135.9 ms, System: 46.6 ms]
  Range (min … max):   583.7 ms … 635.8 ms    10 runs

init
17000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     972.4 ms ±  28.3 ms    [User: 223.4 ms, System: 62.2 ms]
  Range (min … max):   938.4 ms … 1029.7 ms    10 runs
8000
Benchmark 1: typst compile --root . typst-package\examples\bench.typ
  Time (mean ± σ):     687.6 ms ±  20.6 ms    [User: 154.4 ms, System: 24.8 ms]
  Range (min … max):   668.2 ms … 731.7 ms    10 runs

*/
