use super::prelude::*;

#[test]
fn figure() {
    assert_snapshot!(convert_text(r###"\begin{figure}[ht]
        \centering
        \includegraphics[width=0.5\textwidth, height=3cm, angle=45]{example-image.png}
        \caption{This is an example image.}
        \label{fig:example}
    \end{figure}"###).unwrap(), @r###"
    #figure(caption: [This is an example image.],)[

    #image(width: 0.5 * 100%, height: 3cm, "example-image.png")


    ];<fig:example>
    "###);
}

#[test]
fn table() {
    assert_snapshot!(convert_text(r###"\begin{table}[ht]
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
    \end{table}"###).unwrap(), @r###"
    #figure(caption: [This is an example table.],)[

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


    ];<tab:example>
    "###);
}
