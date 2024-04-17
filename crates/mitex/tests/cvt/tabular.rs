use super::prelude::*;

#[test]
fn tabular() {
    assert_debug_snapshot!(convert_text(r###"
    \begin{tabular}{|c|c|}
        \hline
        \textbf{Name} & \textbf{Age} \\
        \hline
        John & 25 \\
        Jane & 22 \\
        \hline
    \end{tabular}
    "###), @r###"
    Ok(
        "\n\n",
    )
    "###);
}
