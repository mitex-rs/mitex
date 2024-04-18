use super::prelude::*;

#[test]
fn figure() {
    assert_snapshot!(convert_text(r###"\begin{figure}[ht]
        \centering
        \includegraphics[width=0.5\textwidth]{example-image}
        \caption{This is an example image.}
        \label{fig:example}
    \end{figure}"###).unwrap(), @r###"

    #figure(caption: [This is an example image.],)[

    #miteximage[\[width=0.5 \]];[example-image];


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




    ];<tab:example>
    "###);
}
