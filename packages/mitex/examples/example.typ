#import "../lib.typ": *

#set page(width: 500pt, height: auto, margin: 1em)

#assert.eq(mitex-convert("\alpha x"), "alpha  x ")

Write inline equations like #mi("x") or #mi[y].

Also block equations (this case is from #text(blue.lighten(20%), link("https://katex.org/")[katex.org])):

#mitex(`
  \newcommand{\f}[2]{#1f(#2)}
  \f\relax{x} = \int_{-\infty}^\infty
    \f\hat\xi\,e^{2 \pi i \xi x}
    \,d\xi
`)

We also support text mode (in development):

#mitex(mode: "text", `
  \section{Title}

  A \textbf{strong} text and a \emph{emph} text.
  
  \begin{enumerate}
    \item This is the first item
    \item This is the second item
    \begin{itemize}
      \item This is the first item
      \item This is the second item
      \item This is the third item
    \end{itemize}
  \end{enumerate}

  Paragraph after itemize and enumerate.
`)
