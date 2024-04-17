use super::prelude::*;

#[test]
fn test_convert_text_mode() {
    assert_debug_snapshot!(convert_text(r#"abc"#), @r###"
    Ok(
        "abc",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\section{Title}"#), @r###"
    Ok(
        "#heading(level: 1)[Title];",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"a \textbf{strong} text"#), @r###"
    Ok(
        "a #strong[strong]; text",
    )
    "###);
    assert_debug_snapshot!(convert_text(r###"
    \section{Title}

    A \textbf{strong} text, a \emph{emph} text and inline equation $x + y$.
    
    Also block \eqref{eq:pythagoras}.
  
    \begin{equation}
      a^2 + b^2 = c^2 \label{eq:pythagoras}
    \end{equation}
    "###), @r###"
    Ok(
        "\n#heading(level: 1)[Title];\n\nA #strong[strong]; text\\, a #emph[emph]; text and inline equation #math.equation(block: false, $x  +  y $);.\n\nAlso block #mitexref[eq:pythagoras];.\n\n$ aligned(\na ^(2 ) +  b ^(2 ) =  c ^(2 ) \n) $<eq:pythagoras>\n",
    )
    "###);
}
