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

#let mitex-color-map = (
  "red": rgb(255, 0, 0),
  "green": rgb(0, 255, 0),
  "blue": rgb(0, 0, 255),
  "cyan": rgb(0, 255, 255),
  "magenta": rgb(255, 0, 255),
  "yellow": rgb(255, 255, 0),
  "black": rgb(0, 0, 0),
  "white": rgb(255, 255, 255),
  "gray": rgb(128, 128, 128),
  "lightgray": rgb(192, 192, 192),
  "darkgray": rgb(64, 64, 64),
  "brown": rgb(165, 42, 42),
  "orange": rgb(255, 165, 0),
  "pink": rgb(255, 182, 193),
  "purple": rgb(128, 0, 128),
  "teal": rgb(0, 128, 128),
  "olive": rgb(128, 128, 0),
)
#let greedy-command(cmd) = (..args) => $cmd(#args.pos().sum())$
#let get-tex-str(tex) = tex.children.filter(it => it != [ ]).map(it => it.text).sum()
#let get-tex-color(texcolor) = {
    mitex-color-map.at(lower(get-tex-str(texcolor)), default: none)
}
#let text-end-space(it) = if it.len() > 1 and it.ends-with(" ") { " " }

#let mitex-scope = (
  negthinspace: h(-(3/18) * 1em),
  negmedspace: h(-(4/18) * 1em),
  negthickspace: h(-(5/18) * 1em),
  enspace: h((1/2) * 1em),
  phantom: hide,
  hphantom: hide,
  vphantom: hide,
  mitexdisplay: greedy-command(math.display),
  mitexinline: greedy-command(math.inline),
  mitexscript: greedy-command(math.script),
  mitexsscript: greedy-command(math.sscript),
  mitexbold: greedy-command(math.bold),
  mitexupright: greedy-command(math.upright),
  mitexitalic: greedy-command(math.italic),
  mitexsans: greedy-command(math.sans),
  mitexfrak: greedy-command(math.frak),
  mitexmono: greedy-command(math.mono),
  mitexcal: greedy-command(math.cal),
  mitexcolor: (texcolor, ..args) => {
    let color = get-tex-color(texcolor)
    if color != none {
      text(fill: color, args.pos().sum())
    } else {
      args.pos().sum()
    }
  },
  colortext: (texcolor, body) => {
    let color = get-tex-color(texcolor)
    if color != none {
      text(fill: get-tex-color(texcolor), body)
    } else {
      body
    }
  },
  colorbox: (texcolor, body) => {
    let color = get-tex-color(texcolor)
    if color != none {
      box(fill: get-tex-color(texcolor), $body$)
    } else {
      body
    }
  },
  frac: (num, den) => $(num)/(den)$,
  cfrac: (num, den) => $display((num)/(den))$,
  dfrac: (num, den) => $display((num)/(den))$,
  tfrac: (num, den) => $inline((num)/(den))$,
  text: it => it,
  textnormal: it => it,
  textbf: it => $bold(it)$ + text-end-space(it),
  textrm: it => $upright(it)$ + text-end-space(it),
  textit: it => $italic(it)$ + text-end-space(it),
  textsf: it => $sans(it)$ + text-end-space(it),
  texttt: it => $mono(it)$ + text-end-space(it),
  mitexbig: it => scale(x: 120%, y: 120%, it),
  mitexBig: it => scale(x: 180%, y: 180%, it),
  mitexbigg: it => scale(x: 240%, y: 240%, it),
  mitexBigg: it => scale(x: 300%, y: 300%, it),
  matrix: math.mat.with(delim: none),
  pmatrix: math.mat.with(delim: "("),
  bmatrix: math.mat.with(delim: "["),
  Bmatrix: math.mat.with(delim: "{"),
  vmatrix: math.mat.with(delim: "|"),
  Vmatrix: math.mat.with(delim: "||"),
  smallmatrix: (..args) => math.inline(math.mat.with(delim: none, ..args)),
  rcases: math.cases.with(reverse: true),
  aligned: it => block(math.op(it)),
  mitexlabel: it => {},
  vspace: it => v(eval(get-tex-str(it))),
  hspace: it => h(eval(get-tex-str(it))),
  stackrel: (sup, base) => $limits(base)^(sup)$,
  overset: (sup, base) => $limits(base)^(sup)$,
  underset: (sub, base) => $limits(base)_(sub)$,
  mitexoverbrace: (it) => math.limits(math.overbrace(it)),
  mitexunderbrace: (it) => math.limits(math.underbrace(it)),
  mitexoverbracket: (it) => math.limits(math.overbracket(it)),
  mitexunderbracket: (it) => math.limits(math.underbracket(it)),
  xleftarrow: it => $limits(xarrow(sym: <--, it))$,
  xrightarrow: it => $limits(xarrow(sym: -->, it))$,
  xLeftarrow: it => $limits(xarrow(sym: <==, it))$,
  xRightarrow: it => $limits(xarrow(sym: ==>, it))$,
  xleftrightarrow: it => $limits(xarrow(sym: <->, it))$,
  xLeftrightarrow: it => $limits(xarrow(sym: <=>, it))$,
  xhookleftarrow: it => $limits(xarrow(sym: -->, it))$,
  xhookrightarrow: it => $limits(xarrow(sym: arrow.l.hook, it))$,
  xtwoheadleftarrow: it => $limits(xarrow(sym: arrow.l.twohead, it))$,
  xtwoheadrightarrow: it => $limits(xarrow(sym: arrow.r.twohead, it))$,
  xleftharpoonup: it => $limits(xarrow(sym: harpoon.lt, it))$,
  xrightharpoonup: it => $limits(xarrow(sym: harpoon.rt, it))$,
  xleftharpoondown: it => $limits(xarrow(sym: harpoon.lb, it))$,
  xrightharpoondown: it => $limits(xarrow(sym: harpoon.rb, it))$,
  xleftrightharpoons: it => $limits(xarrow(sym: harpoons.ltrb, it))$,
  xrightleftharpoons: it => $limits(xarrow(sym: harpoons.rtlb, it))$,
  xtofrom: it => $limits(xarrow(sym: arrows.rl, it))$,
  xmapsto: it => $limits(xarrow(sym: |->, it))$,
  xlongequal: it => $limits(xarrow(sym: eq, it))$,
  pmod: it => $quad (mod thick it)$,
  operatorname: it => math.op(math.upright(it)),
  operatornamewithlimits: it => math.op(limits: true, math.upright(it)),
  mitexarray: (arg0: ("l",), ..args) => {
    if args.pos().len() == 0 {
      return
    }
    if type(arg0) != str {
      if arg0.has("children") {
        arg0 = arg0.children.filter(it => it != [ ])
          .map(it => it.text)
          .filter(it => it == "l" or it == "c" or it == "r")
      } else {
        arg0 = (arg0.text,)
      }
    }
    let matrix = args.pos().map(row => if type(row) == array { row } else { (row,) } )
    let n = matrix.len()
    let m = calc.max(..matrix.map(row => row.len()))
    matrix = matrix.map(row => row + (m - row.len()) * (none,))
    let array-at(arr, pos) = {
      arr.at(calc.min(pos, arr.len() - 1))
    }
    let align-map = ("l": left, "c": center, "r": right)
    set align(align-map.at(array-at(arg0, 0)))
    pad(y: 0.2em, grid(
      columns: m,
      column-gutter: 0.5em,
      row-gutter: 0.5em,
      ..matrix.flatten().map(it => $it$)
    ))
  },
  mitexsqrt: (..args) => {
    if args.pos().len() == 1 {
      $sqrt(#args.pos().at(0))$
    } else if args.pos().len() == 2 {
      $root(
        #args.pos().at(0).children.filter(it => it != [\[] and it != [\]]).sum(),
        #args.pos().at(1)
      )$
    } else {
      panic("unexpected args in sqrt")
    }
  },
)

#let mitex(it, block: true) = {
  let res = mitex-convert(it)
  math.equation(block: block, eval("$" + res + "$", scope: mitex-scope))
}

#let mi = mitex.with(block: false)
