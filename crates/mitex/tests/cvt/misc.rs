use super::prelude::*;

#[test]
fn test_convert_word() {
    assert_debug_snapshot!(convert_math(r#"$abc$"#), @r###"
    Ok(
        "a b c ",
    )
    "###);
}

#[test]
fn test_convert_greek() {
    assert_debug_snapshot!(convert_math(r#"$\alpha x$"#), @r###"
    Ok(
        "alpha  x ",
    )
    "###);
}

#[test]
fn test_convert_command() {
    assert_debug_snapshot!(convert_math(r#"$\int_1^2 x \mathrm{d} x$"#), @r###"
    Ok(
        "integral _(1 )^(2 ) x  upright(d ) x ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$\underline{T}$"#), @r###"
    Ok(
        "underline(T )",
    )
    "###);
}

#[test]
fn test_convert_frac() {
    assert_debug_snapshot!(convert_math(r#"$\frac{a}{b}$"#), @r###"
    Ok(
        "frac(a ,b )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\frac 12_3$"#), @r###"
    Ok(
        "frac(1 ,2 )_(3 )",
    )
    "###
    );
    // Note: the following is invalid in TeX, hence we may output anything.
    let _ = convert_math(r#"$\frac a_c b$"#);
}

#[test]
fn test_convert_displaystyle() {
    assert_debug_snapshot!(convert_math(r#"$\displaystyle xyz\frac{1}{2}$"#), @r###"
    Ok(
        "mitexdisplay( x y z frac(1 ,2 ))",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$1 + {\displaystyle 23} + 4$"#), @r###"
    Ok(
        "1  +  mitexdisplay( 2 3 ) +  4 ",
    )
    "###
    );
}

#[test]
fn test_convert_limits() {
    assert_debug_snapshot!(convert_math(r#"$\sum\limits_1^2$"#), @r###"
    Ok(
        "limits(sum )_(1 )^(2 )",
    )
    "###
    );
}

#[test]
fn test_convert_subsup() {
    assert_debug_snapshot!(convert_math(r#"$x_1^2$"#), @r###"
    Ok(
        "x _(1 )^(2 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$x^2_1$"#), @r###"
    Ok(
        "x ^(2 )_(1 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$x''_1$"#), @r###"
    Ok(
        "x ''_(1 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\overbrace{a + b + c}^{\text{This is an overbrace}}$"#), @r###"
    Ok(
        "mitexoverbrace(a  +  b  +  c )^(#textmath[This is an overbrace];)",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$x_1''$"#), @r###"
    Ok(
        "x _(1 )''",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"${}_1^2x_3^4$"#), @r###"
    Ok(
        "zws _(1 )^(2 )x _(3 )^(4 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$_1^2$"#), @r###"
    Ok(
        "zws_(1 )zws^(2 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\frac{_1^2}{3}$"#), @r###"
    Ok(
        "frac(zws_(1 )zws^(2 ),3 )",
    )
    "###
    );
}

#[test]
fn test_convert_over() {
    assert_debug_snapshot!(convert_math(r#"$x + 1 \over y + 2$"#), @r###"
    Ok(
        "frac(x  +  1  , y  +  2 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$1 + {2 \over 3}$"#), @r###"
    Ok(
        "1  +  frac(2  , 3 )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"${l \over 2'}$"#), @r###"
    Ok(
        "frac(l  , 2 ')",
    )
    "###);
}

#[test]
fn test_convert_divide() {
    assert_debug_snapshot!(convert_math(r#"$x / y$"#), @r###"
    Ok(
        "x  \\/ y ",
    )
    "###
    );
}

#[test]
fn test_convert_space() {
    assert_debug_snapshot!(convert_math(r#"$x~\! \, \> \: \; \ \quad \qquad y$"#), @r###"
    Ok(
        "x space.nobreak negthinspace  thin  med  med  thick  thick  quad  wide  y ",
    )
    "###
    );
}

#[test]
fn test_convert_escape() {
    assert_debug_snapshot!(convert_math(r#"$\|x\|| \& \# \% \$ y$"#), @r###"
    Ok(
        "|| x || |  amp  hash  percent  dollar  y ",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$a*b * c$"#).unwrap(), @r###""a \\*b  \\* c ""###
    );
    assert_debug_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###""\\\"""###
    );
    assert_debug_snapshot!(convert_text(r#"@abc"#).unwrap(), @r###""\\@abc""###
    );
}

#[test]
fn test_unreachable() {
    // println!("{:#?}", convert_math(r#"$u^-$"#));
    assert_debug_snapshot!(convert_math(r#"$u^−$"#).unwrap(), @r###""u ^(− )""###
    );
}

#[test]
fn test_convert_sqrt() {
    assert_debug_snapshot!(convert_math(r#"$\sqrt 1$"#), @r###"
    Ok(
        "mitexsqrt(1 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$\sqrt [1]2$"#), @r###"
    Ok(
        "mitexsqrt(\\[1 \\],2 )",
    )
    "###
    );
}

#[test]
fn test_convert_lr() {
    assert_debug_snapshot!(convert_math(r#"$\left.\right.$"#), @r###"
    Ok(
        "lr(  )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$\left.a\right.$"#), @r###"
    Ok(
        "lr( a  )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$\alpha\left.\right.$"#), @r###"
    Ok(
        "alpha lr(  )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\left  . a \right    \|$"#), @r###"
    Ok(
        "lr(     a       || )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\left\langle a\right\|$"#), @r###"
    Ok(
        "lr(angle.l  a || )",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$\left\lbrack\lbrack x\rbrack\right\rbrack$"#), @r###"
    Ok(
        "lr(bracket.l bracket.l  x bracket.r bracket.r )",
    )
    "###
    );
}

#[test]
fn test_convert_color() {
    assert_debug_snapshot!(convert_math(r#"$x\color{red}yz\frac{1}{2}$"#), @r###"
    Ok(
        "x mitexcolor(r e d ,y z frac(1 ,2 ))",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}yz$"#), @r###"
    Ok(
        "x colortext(r e d ,y )z ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}{yz}$"#), @r###"
    Ok(
        "x colortext(r e d ,y z )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}yz$"#), @r###"
    Ok(
        "x colorbox(r e d ,y )z ",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}{yz}$"#), @r###"
    Ok(
        "x colorbox(r e d ,y z )",
    )
    "###
    );
}

#[test]
fn test_convert_matrix() {
    assert_debug_snapshot!(convert_math(
                 r#"$\begin{pmatrix}x{\\}x\end{pmatrix}$"#
        ).unwrap(),
        @r###""pmatrix(x x )""###
    );
    assert_debug_snapshot!(convert_math(
                 r#"$\begin{pmatrix} \\ & \ddots \end{pmatrix}$"#
        ).unwrap(),
        @r###""pmatrix( zws ; zws , dots.down  )""###
    );
    assert_debug_snapshot!(convert_math(
            r#"$\begin{matrix}
    1 & 2 & 3\\
a & b & c
\end{matrix}$"#
        ),
        @r###"
    Ok(
        "matrix(\n1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(
            r#"$\begin{Vmatrix}
    1 & 2 & 3\\
a & b & c
\end{Vmatrix}$"#
        ),
        @r###"
    Ok(
        "Vmatrix(\n1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(
            r#"$\begin{array}{lcr}
    1 & 2 & 3\\
a & b & c
\end{array}$"#
        ),
        @r###"
    Ok(
        "mitexarray(arg0: l c r ,\n1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
    )
    "###
    );
}

#[test]
fn test_convert_env() {
    assert_debug_snapshot!(convert_math(
            r#"$\begin{aligned}
    1 & 2 & 3\\
a & b & c
\end{aligned}$"#
        ),
        @r###"
    Ok(
        "aligned(\n1  & 2  & 3 \\ \na  & b  & c \n)",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(
            r#"$\begin{align*}
    1 & 2 & 3\\
a & b & c
\end{align*}$"#
        ),
        @r###"
    Ok(
        "aligned(\n1  & 2  & 3 \\ \na  & b  & c \n)",
    )
    "###
    );
    assert_debug_snapshot!(convert_math(
            r#"$\begin{cases}
    1 & 2 & 3\\
a & b & c
\end{cases}$"#
        ),
        @r###"
    Ok(
        "cases(\n1  & 2  & 3 ,\na  & b  & c \n)",
    )
    "###
    );
}

#[test]
fn test_convert_ditto() {
    assert_debug_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###""\\\"""###);
    assert_debug_snapshot!(convert_math(r#"$a"b"c$"#).unwrap(), @r###""a \\\"b \\\"c ""###);
    assert_debug_snapshot!(convert_math(r#"$\text{a"b"c}$"#).unwrap(), @r###""#textmath[a\\\"b\\\"c];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{a " b " c}$"#).unwrap(), @r###""#textmath[a \\\" b \\\" c];""###);
}

#[test]
fn test_convert_text() {
    assert_debug_snapshot!(convert_math(r#"$\text{abc}$"#).unwrap(), @r###""#textmath[abc];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ a b c }$"#).unwrap(), @r###""#textmath[ a b c ];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{abc{}}$"#).unwrap(), @r###""#textmath[abc];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab{}c}$"#).unwrap(), @r###""#textmath[abc];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab c}$"#).unwrap(), @r###""#textmath[ab c];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab$x$c}$"#).unwrap(), @r###""#textmath[ab#math.equation(block: false, $x $);c];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab*c}$"#).unwrap(), @r###""#textmath[ab\\*c];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab_c}$"#).unwrap(), @r###""#textmath[ab\\_c];""###);
    assert_debug_snapshot!(convert_math(r#"$\text{ab^c}$"#).unwrap(), @r###""#textmath[ab\\^c];""###);
    // note: hack doesn't work in this case
    assert_debug_snapshot!(convert_math(r#"$\text{ab\color{red}c}$"#).unwrap(), @r###""#textmath[abmitexcolor(red,c)];""###);
}

#[test]
fn test_convert_typst_code() {
    assert_snapshot!(convert_math(r#"\iftypst#show: template\fi"#).unwrap(), @"#show: template");
    assert_snapshot!(convert_math(r#"\iftypst#import "template.typ": project
#show: project\fi"#).unwrap(), @r###"
    #import "template.typ": project
    #show: project
    "###);
}

#[test]
fn test_convert_formula() {
    assert_debug_snapshot!(convert_text(r#"$a$"#), @r###"
    Ok(
        "#math.equation(block: false, $a $);",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"$$a$$"#), @r###"
    Ok(
        "$ a  $",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\(a\)"#), @r###"
    Ok(
        "#math.equation(block: false, $a $);",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\[a\]"#), @r###"
    Ok(
        "$ a  $",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"$ a $"#), @r###"
    Ok(
        "#math.equation(block: false, $ a  $);",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"$$ a $ $ b $$"#), @r###"
    Err(
        "error: formula is not valid",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\[a\)\(b\]"#), @r###"
    Err(
        "error: formula is not valid",
    )
    "###);
}

#[test]
fn test_fuzzing() {
    assert_debug_snapshot!(convert_math(r#"\left\0"#).unwrap_err(), @r###""error: unknown command: \\0""###);
    assert_debug_snapshot!(convert_math(r#"\end{}"#).unwrap_err(), @r###""error: error unexpected: \"\"""###);
}
