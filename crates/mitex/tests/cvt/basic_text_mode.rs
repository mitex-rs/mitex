use super::prelude::*;

#[test]
fn test_convert_text_mode() {
    assert_snapshot!(convert_text(r#"abc"#).unwrap(), @"abc");
    assert_snapshot!(convert_text(r#"\section{Title}"#).unwrap(), @"#heading(level: 1)[Title];");
    assert_snapshot!(convert_text(r#"a \textbf{strong} text"#).unwrap(), @"a #strong[strong]; text");
    assert_snapshot!(convert_text(r###"\section{Title}

    A \textbf{strong} text, a \emph{emph} text and inline equation $x + y$.
    
    Also block \eqref{eq:pythagoras}.
  
    \begin{equation}
      a^2 + b^2 = c^2 \label{eq:pythagoras}
    \end{equation}"###).unwrap(), @r###"

    #heading(level: 1)[Title];

    A #strong[strong]; text\, a #emph[emph]; text and inline equation #math.equation(block: false, $x  +  y $);.

    Also block #mitexref[eq:pythagoras];.

    $ aligned(
    a ^(2 ) +  b ^(2 ) =  c ^(2 ) 
    ) $<eq:pythagoras>
    "###);
}
