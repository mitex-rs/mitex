#let mitex-wasm = plugin("./mitex.wasm")

#let mitex-convert(it) = {
  str(mitex-wasm.convert_math(bytes("$" + {
    if type(it) == str {
      it
    } else if type(it) == content and it.has("text") {
      it.text
    } else {
      panic("Unsupported type: " + str(type(it)))
    }
  } + "$")))
}

#let greedy-command(cmd) = (..args) => $cmd(#args.pos().sum())$

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
#let get-tex-str(tex) = tex.children.filter(it => it != [ ]).map(it => it.text).sum()
#let get-tex-color(texcolor) = {
    mitex-color-map.at(get-tex-str(texcolor), default: black)
}

#let mitex-scope = (
  negativespace: h(-(1/6) * 1em),
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
  color: (texcolor, ..args) => text(fill: get-tex-color(texcolor), args.pos().sum()),
  textcolor: (texcolor, body) => text(fill: get-tex-color(texcolor), body),
  colorbox: (texcolor, body) => box(fill: get-tex-color(texcolor), $body$),
  frac: (num, den) => $(num)/(den)$,
  cfrac: (num, den) => $display((num)/(den))$,
  dfrac: (num, den) => $display((num)/(den))$,
  tfrac: (num, den) => $inline((num)/(den))$,
  matrix: math.mat.with(delim: none),
  pmatrix: math.mat.with(delim: "("),
  bmatrix: math.mat.with(delim: "["),
  Bmatrix: math.mat.with(delim: "{"),
  vmatrix: math.mat.with(delim: "|"),
  Vmatrix: math.mat.with(delim: "||"),
  mitexarray: (arg0: none, ..args) => math.mat(delim: none, ..args),
  aligned: it => block(math.op(it)),
  mitexlabel: it => {},
  vspace: it => v(eval(get-tex-str(it))),
  hspace: it => h(eval(get-tex-str(it))),
  stackrel: (sup, base) => $limits(base)^(sup)$,
  overset: (sup, base) => $limits(base)^(sup)$,
  underset: (sub, base) => $limits(base)_(sub)$,
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
  if block {
    eval("$ " + res + " $", scope: mitex-scope)
  } else {
    eval("$" + res + "$", scope: mitex-scope)
  }
}

#let mi = mitex.with(block: false)
