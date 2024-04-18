use super::prelude::*;

#[test]
fn tabular() {
    assert_snapshot!(convert_text(r###"\begin{tabular}{|c|c|}
        \hline
        \textbf{Name} & \textbf{Age} \\
        \hline
        John & 25 \\
        Jane & 22 \\
        \hline
    \end{tabular}"###).unwrap(), @r###"
    #table(stroke: none,
    columns: 2,
    align: (center, center, ),
    table.vline(stroke: .5pt, x: 0), table.vline(stroke: .5pt, x: 1), table.vline(stroke: .5pt, x: 2), 
    table.hline(stroke: .5pt),
    [#strong[Name]; ], [#strong[Age]; ],
    table.hline(stroke: .5pt),
    [John ], [25 ],
    [Jane ], [22 ],
    table.hline(stroke: .5pt),
    );
    "###);
}
