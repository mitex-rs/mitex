use super::prelude::*;

#[test]
fn easy() {
    assert_debug_snapshot!(convert_text(r#"\begin{equation}\end{equation}"#), @r###"
    Ok(
        "$ aligned() $",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\begin{equation}\end{equation}"#), @r###"
    Ok(
        "aligned()",
    )
    "###);
}

#[test]
fn matrix() {
    assert_debug_snapshot!(convert_math(
            r#"\begin{matrix}
a & b \\
c & d
\end{matrix}"#), @r###"
    Ok(
        "matrix(\na  zws , b  zws ;\nc  zws , d \n)",
    )
    "###);
    assert_debug_snapshot!(convert_math(
            r#"\begin{pmatrix}\\\end{pmatrix}"#), @r###"
    Ok(
        "pmatrix(zws ;)",
    )
    "###);
    assert_debug_snapshot!(convert_math(
            r#"\begin{pmatrix}x{\\}x\end{pmatrix}"#), @r###"
    Ok(
        "pmatrix(x x )",
    )
    "###);
}

#[test]
fn arguments() {
    assert_debug_snapshot!(convert_math(
            r#"\begin{array}{lc}
a & b \\
c & d
\end{array}"#), @r###"
    Ok(
        "mitexarray(arg0: l c ,\na  zws , b  zws ;\nc  zws , d \n)",
    )
    "###);
}

#[test]
fn space_around_and() {
    assert_debug_snapshot!(convert_math(
            r#"\begin{bmatrix}A&B\end{bmatrix}"#), @r###"
    Ok(
        "bmatrix(A zws ,B )",
    )
    "###);
}
