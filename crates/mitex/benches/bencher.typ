
#import "/packages/mitex/lib.typ": *

#let integrate-conversion(it, data: (), convert-only: false) = {
  let passed = 0
  for d in data {

    passed += 1

    if convert-only {
      let _ = mitex-convert(d.text)
    } else /* render-math */ {
      if d.type == "inline" {
        mi(d.text)
        linebreak()
      } else {
        mitex(d.text)
      }
    }
  }

  it
  [#passed / #data.len() passed]
}
