// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(title: "", authors: (), body) = {
  // Set the document's basic properties.
  set document(author: authors, title: title)
  set page(
    height: auto,
    width: 210mm,
    // numbering: "1", number-align: center,
  )
  set text(font: ("Linux Libertine", "Source Han Sans"), size: 14pt, lang: "en")
  // set page(height: 297mm)


  // Title row.
  align(center)[
    #block(text(weight: 700, 1.75em, title))
  ]

  // Main body.
  set par(justify: true)

  show raw.where(block: true): rect.with(width: 100%, radius: 2pt, fill: luma(240), stroke: 0pt)

  body
}
