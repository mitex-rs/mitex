
#let wh = 2.5em
#set page(width: wh, height: wh, margin: .08em)

// Default font of Typst
#set text(font: "Linux Libertine")

#let TeX = {
// Default font of LaTeX
  set text(font: "New Computer Modern", weight: "regular")
  box(width: 1.7em, {
    [T]
    place(top, dx: 0.56em, dy: 0.22em)[E]
    place(top, dx: 1.1em)[X]
  })
}

#let func = text.with(fill: rgb("4b69c6"))
#let punc = text.with(fill: rgb("d73a49"))
#align(left + horizon, {
  v(-0.1em)
  box(scale(61.8%, func("mi") + punc("\u{005B}")))
  linebreak() + v(-1.1em)
  h(0.35em) + TeX
  linebreak() + v(-1.2em)
  box(scale(61.8%, [~]+punc("\u{005D}")))
})
