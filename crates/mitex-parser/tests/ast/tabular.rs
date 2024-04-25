use super::prelude::*;

#[test]
fn tabular() {
    assert_debug_snapshot!(parse(r###"
    \begin{tabular}{|c|c|}
        \hline
        \textbf{Name} & \textbf{Age} \\
        \hline
        John & 25 \\
        Jane & 22 \\
        \hline
    \end{tabular}
    "###), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||sym'("tabular")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("|c|c|"))
    |||||rbrace'("}")
    ||br'("\n")
    ||space'("        ")
    ||cmd(cmd-name("\\hline"))
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\textbf")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("Name"))
    |||||rbrace'("}")
    ||space'(" ")
    ||ampersand'("&")
    ||space'(" ")
    ||cmd
    |||cmd-name("\\textbf")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("Age"))
    |||||rbrace'("}")
    ||space'(" ")
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||cmd(cmd-name("\\hline"))
    ||br'("\n")
    ||space'("        ")
    ||text(word'("John"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("25"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||text(word'("Jane"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("22"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||cmd(cmd-name("\\hline"))
    ||br'("\n")
    ||space'("    ")
    ||end(sym'("tabular"))
    |br'("\n")
    |space'("    ")
    "###);
}
