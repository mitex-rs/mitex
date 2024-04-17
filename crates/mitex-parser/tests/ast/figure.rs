use super::prelude::*;

#[test]
fn figure() {
    assert_debug_snapshot!(parse(r###"
    \begin{figure}[ht]
        \centering
        \includegraphics[width=0.5\textwidth]{example-image}
        \caption{This is an example image.}
        \label{fig:example}
    \end{figure}
    "###), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||sym'("figure")
    |||args
    ||||bracket
    |||||lbracket'("[")
    |||||text(word'("ht"))
    |||||rbracket'("]")
    ||br'("\n")
    ||space'("        ")
    ||cmd(cmd-name("\\centering"))
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\includegraphics")
    |||args
    ||||bracket
    |||||lbracket'("[")
    |||||text(word'("width=0.5"))
    |||||cmd(cmd-name("\\textwidth"))
    |||||rbracket'("]")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("example-image"))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\caption")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("This"),space'(" "),word'("is"),space'(" "),word'("an"),space'(" "),word'("example"),space'(" "),word'("image."))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\label")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("fig:example"))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("    ")
    ||end(sym'("figure"))
    |br'("\n")
    |space'("    ")
    "###);
}

#[test]
fn table() {
    assert_debug_snapshot!(parse(r###"
    \begin{table}[ht]
        \centering
        \begin{tabular}{|c|c|}
            \hline
            \textbf{Name} & \textbf{Age} \\
            \hline
            John & 25 \\
            Jane & 22 \\
            \hline
        \end{tabular}
        \caption{This is an example table.}
        \label{tab:example}
    \end{table}
    "###), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||sym'("table")
    |||args
    ||||bracket
    |||||lbracket'("[")
    |||||text(word'("ht"))
    |||||rbracket'("]")
    ||br'("\n")
    ||space'("        ")
    ||cmd(cmd-name("\\centering"))
    ||br'("\n")
    ||space'("        ")
    ||env
    |||begin
    ||||sym'("tabular")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("|c|c|"))
    ||||||rbrace'("}")
    |||br'("\n")
    |||space'("            ")
    |||cmd(cmd-name("\\hline"))
    |||br'("\n")
    |||space'("            ")
    |||cmd
    ||||cmd-name("\\textbf")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("Name"))
    ||||||rbrace'("}")
    |||space'(" ")
    |||ampersand'("&")
    |||space'(" ")
    |||cmd
    ||||cmd-name("\\textbf")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("Age"))
    ||||||rbrace'("}")
    |||space'(" ")
    |||newline("\\\\")
    |||br'("\n")
    |||space'("            ")
    |||cmd(cmd-name("\\hline"))
    |||br'("\n")
    |||space'("            ")
    |||text(word'("John"),space'(" "))
    |||ampersand'("&")
    |||space'(" ")
    |||text(word'("25"),space'(" "))
    |||newline("\\\\")
    |||br'("\n")
    |||space'("            ")
    |||text(word'("Jane"),space'(" "))
    |||ampersand'("&")
    |||space'(" ")
    |||text(word'("22"),space'(" "))
    |||newline("\\\\")
    |||br'("\n")
    |||space'("            ")
    |||cmd(cmd-name("\\hline"))
    |||br'("\n")
    |||space'("        ")
    |||end(sym'("tabular"))
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\caption")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("This"),space'(" "),word'("is"),space'(" "),word'("an"),space'(" "),word'("example"),space'(" "),word'("table."))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\label")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("tab:example"))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("    ")
    ||end(sym'("table"))
    |br'("\n")
    |space'("    ")
    "###);
}
