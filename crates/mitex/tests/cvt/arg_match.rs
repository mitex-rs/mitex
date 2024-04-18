use super::prelude::*;

#[test]
fn curly_group() {
    assert_snapshot!(convert_math(r#"a \textbf{strong} text"#).unwrap(), @"a  #textbf[strong]; t e x t ");
    assert_snapshot!(convert_math(r#"x \color {red} yz \frac{1}{2}"#).unwrap(), @"x  mitexcolor( r e d , y z  frac(1 ,2 ))");
}

#[test]
fn split_char() {
    assert_snapshot!(convert_math(r#"\frac abcd"#).unwrap(), @"frac(a ,b )c d ");
    assert_snapshot!(convert_math(r#"\frac ab"#).unwrap(), @"frac(a ,b )");
    assert_snapshot!(convert_math(r#"\frac a"#).unwrap(), @"frac(a )");
}

#[test]
fn eat_regular_brace() {
    assert_snapshot!(convert_math(r#"\mathrm(x)"#).unwrap(), @r###"upright(\()x \)"###);
    assert_snapshot!(convert_math(r#"\mathrm[x]"#).unwrap(), @r###"upright(\[)x \]"###);
    assert_snapshot!(convert_math(r#"\mathrm\lbrace x \rbrace"#).unwrap(), @r###"upright(\{ ) x  \} "###);
}

#[test]
fn special_marks() {
    // & and newline'
    assert_snapshot!(convert_math(r#"\begin{matrix}
        \displaystyle 1 & 2 \\
        3 & 4 \\
    \end{matrix}"#).unwrap(), @r###"

    matrix(
    mitexdisplay( 1  )zws , 2  zws ;
    3  zws , 4  zws ;
    )
    "###);
    assert_snapshot!(convert_math(r#"\begin{matrix}
        \displaystyle 1 \\
        3 \\
    \end{matrix}"#).unwrap(), @r###"

    matrix(
    mitexdisplay( 1  )zws ;
    3  zws ;
    )
    "###);
    assert_snapshot!(convert_math(r#"\begin{matrix}\frac{1} & {2}\end{matrix}"#).unwrap(), @r###"

    matrix(frac(1 ) zws , 2 )
    "###);
    assert_snapshot!(convert_math(r#"\begin{matrix}\frac{1} \\ {2}\end{matrix}"#).unwrap(), @r###"

    matrix(frac(1 ,zws ;) 2 )
    "###);
    assert_snapshot!(convert_math(r#"1 \over 2 \\ 3 "#).unwrap(), @r###"frac(1  , 2  \  3  )"###);
}

#[test]
fn special_marks_in_env() {
    assert_snapshot!(convert_math(r#"\displaystyle \frac{1}{2} \\ \frac{1}{2}"#).unwrap(), @r###"mitexdisplay( frac(1 ,2 ) \  frac(1 ,2 ))"###);
    assert_snapshot!(convert_math(r#"\left. \displaystyle \frac{1}{2} \\ \frac{1}{2} \right."#).unwrap(), @r###"

    lr(  mitexdisplay( frac(1 ,2 ) \  frac(1 ,2 ) ) )
    "###);
    assert_snapshot!(convert_math(r#"\sqrt[\displaystyle \frac{1}{2} \\ \frac{1}{2} ]{}"#).unwrap(), @r###"

    mitexsqrt(\[mitexdisplay( frac(1 ,2 ) \  frac(1 ,2 ) )\],zws )
    "###);
    assert_snapshot!(convert_math(r#"\begin{matrix}a \over b \\ c\end{matrix}"#).unwrap(), @r###"

    matrix(frac(a  , b  )zws ; c )
    "###);
}

#[test]
fn sqrt_pattern() {
    assert_snapshot!(convert_math(r#"\sqrt 12"#).unwrap(), @"mitexsqrt(1 )2 ");
    assert_snapshot!(convert_math(r#"\sqrt{1}2"#).unwrap(), @"mitexsqrt(1 )2 ");
    // Note: this is an invalid expression
    assert_snapshot!(convert_math(r#"\sqrt[1]"#).unwrap(), @r###"mitexsqrt(\[1 \])"###);
    assert_snapshot!(convert_math(r#"\sqrt[1]{2}"#).unwrap(), @r###"mitexsqrt(\[1 \],2 )"###);
    assert_snapshot!(convert_math(r#"\sqrt[1]{2}3"#).unwrap(), @r###"mitexsqrt(\[1 \],2 )3 "###);
}
