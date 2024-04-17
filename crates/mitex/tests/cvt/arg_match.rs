use super::prelude::*;

#[test]
fn curly_group() {
    assert_debug_snapshot!(convert_math(r#"a \textbf{strong} text"#), @r###"
    Ok(
        "a  #textbf[strong]; t e x t ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"x \color {red} yz \frac{1}{2}"#), @r###"
    Ok(
        "x  mitexcolor( r e d , y z  frac(1 ,2 ))",
    )
    "###);
}

#[test]
fn split_char() {
    assert_debug_snapshot!(convert_math(r#"\frac abcd"#), @r###"
    Ok(
        "frac(a ,b )c d ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac ab"#), @r###"
    Ok(
        "frac(a ,b )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac a"#), @r###"
    Ok(
        "frac(a )",
    )
    "###);
}

#[test]
fn eat_regular_brace() {
    assert_debug_snapshot!(convert_math(r#"\mathrm(x)"#), @r###"
    Ok(
        "upright(\\()x \\)",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\mathrm[x]"#), @r###"
    Ok(
        "upright(\\[)x \\]",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\mathrm\lbrace x \rbrace"#), @r###"
    Ok(
        "upright(\\{ ) x  \\} ",
    )
    "###);
}

#[test]
fn special_marks() {
    // & and newline'
    assert_debug_snapshot!(convert_math(r#"
    \begin{matrix}
        \displaystyle 1 & 2 \\
        3 & 4 \\
    \end{matrix}
    "#), @r###"
    Ok(
        "\nmatrix(\nmitexdisplay( 1  )zws , 2  zws ;\n3  zws , 4  zws ;\n)\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \begin{matrix}
        \displaystyle 1 \\
        3 \\
    \end{matrix}
    "#), @r###"
    Ok(
        "\nmatrix(\nmitexdisplay( 1  )zws ;\n3  zws ;\n)\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \begin{matrix}\frac{1} & {2}\end{matrix}
    "#), @r###"
    Ok(
        "\nmatrix(frac(1 ) zws , 2 )\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \begin{matrix}\frac{1} \\ {2}\end{matrix}
    "#), @r###"
    Ok(
        "\nmatrix(frac(1 ,zws ;) 2 )\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    1 \over 2 \\ 3 
    "#), @r###"
    Ok(
        "frac(\n1  , 2  \\  3  \n)",
    )
    "###);
}

#[test]
fn special_marks_in_env() {
    assert_debug_snapshot!(convert_math(r#"
    \displaystyle \frac{1}{2} \\ \frac{1}{2} 
    "#), @r###"
    Ok(
        "\nmitexdisplay( frac(1 ,2 ) \\  frac(1 ,2 ) \n)",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \left. \displaystyle \frac{1}{2} \\ \frac{1}{2} \right.
    "#), @r###"
    Ok(
        "\nlr(  mitexdisplay( frac(1 ,2 ) \\  frac(1 ,2 ) ) )\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \sqrt[\displaystyle \frac{1}{2} \\ \frac{1}{2} ]{}
    "#), @r###"
    Ok(
        "\nmitexsqrt(\\[mitexdisplay( frac(1 ,2 ) \\  frac(1 ,2 ) )\\],zws )\n",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"
    \begin{matrix}a \over b \\ c\end{matrix}
    "#), @r###"
    Ok(
        "\nmatrix(frac(a  , b  )zws ; c )\n",
    )
    "###);
}

#[test]
fn sqrt_pattern() {
    assert_debug_snapshot!(convert_math(r#"\sqrt 12"#), @r###"
    Ok(
        "mitexsqrt(1 )2 ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sqrt{1}2"#), @r###"
    Ok(
        "mitexsqrt(1 )2 ",
    )
    "###);
    // Note: this is an invalid expression
    assert_debug_snapshot!(convert_math(r#"\sqrt[1]"#), @r###"
    Ok(
        "mitexsqrt(\\[1 \\])",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sqrt[1]{2}"#), @r###"
    Ok(
        "mitexsqrt(\\[1 \\],2 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sqrt[1]{2}3"#), @r###"
    Ok(
        "mitexsqrt(\\[1 \\],2 )3 ",
    )
    "###);
}
