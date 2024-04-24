use super::prelude::*;

#[test]
fn test_convert_word() {
    assert_snapshot!(convert_math(r#"$abc$"#).unwrap(), @"a b c ");
}

#[test]
fn test_convert_greek() {
    assert_snapshot!(convert_math(r#"$\alpha x$"#).unwrap(), @"alpha  x ");
}

#[test]
fn test_convert_command() {
    assert_snapshot!(convert_math(r#"$\int_1^2 x \mathrm{d} x$"#).unwrap(), @"integral _(1 )^(2 ) x  upright(d ) x ");
    assert_snapshot!(convert_math(r#"$\underline{T}$"#).unwrap(), @"underline(T )");
}

#[test]
fn test_convert_frac() {
    assert_snapshot!(convert_math(r#"$\frac{a}{b}$"#).unwrap(), @"frac(a ,b )"
    );
    assert_snapshot!(convert_math(r#"$\frac 12_3$"#).unwrap(), @"frac(1 ,2 )_(3 )"
    );
    // Note: the following is invalid in TeX, hence we may output anything.
    let _ = convert_math(r#"$\frac a_c b$"#);
}

#[test]
fn test_convert_displaystyle() {
    assert_snapshot!(convert_math(r#"$\displaystyle xyz\frac{1}{2}$"#).unwrap(), @"mitexdisplay( x y z frac(1 ,2 ))"
    );
    assert_snapshot!(convert_math(r#"$1 + {\displaystyle 23} + 4$"#).unwrap(), @"1  +  mitexdisplay( 2 3 ) +  4 "
    );
}

#[test]
fn test_convert_limits() {
    assert_snapshot!(convert_math(r#"$\sum\limits_1^2$"#).unwrap(), @"limits(sum )_(1 )^(2 )"
    );
}

#[test]
fn test_convert_subsup() {
    assert_snapshot!(convert_math(r#"$x_1^2$"#).unwrap(), @"x _(1 )^(2 )"
    );
    assert_snapshot!(convert_math(r#"$x^2_1$"#).unwrap(), @"x ^(2 )_(1 )"
    );
    assert_snapshot!(convert_math(r#"$x''_1$"#).unwrap(), @"x ''_(1 )"
    );
    assert_snapshot!(convert_math(r#"$\overbrace{a + b + c}^{\text{This is an overbrace}}$"#).unwrap(), @"mitexoverbrace(a  +  b  +  c )^(#textmath[This is an overbrace];)"
    );
    assert_snapshot!(convert_math(r#"$x_1''$"#).unwrap(), @"x _(1 )''"
    );
    assert_snapshot!(convert_math(r#"${}_1^2x_3^4$"#).unwrap(), @"zws _(1 )^(2 )x _(3 )^(4 )"
    );
    assert_snapshot!(convert_math(r#"$_1^2$"#).unwrap(), @"zws_(1 )zws^(2 )"
    );
    assert_snapshot!(convert_math(r#"$\frac{_1^2}{3}$"#).unwrap(), @"frac(zws_(1 )zws^(2 ),3 )"
    );
}

#[test]
fn test_convert_over() {
    assert_snapshot!(convert_math(r#"$x + 1 \over y + 2$"#).unwrap(), @"frac(x  +  1  , y  +  2 )"
    );
    assert_snapshot!(convert_math(r#"$1 + {2 \over 3}$"#).unwrap(), @"1  +  frac(2  , 3 )"
    );
    assert_snapshot!(convert_math(r#"${l \over 2'}$"#).unwrap(), @"frac(l  , 2 ')");
}

#[test]
fn test_convert_divide() {
    assert_snapshot!(convert_math(r#"$x / y$"#).unwrap(), @r###"x  \/ y "###
    );
}

#[test]
fn test_convert_space() {
    assert_snapshot!(convert_math(r#"$x~\! \, \> \: \; \ \quad \qquad y$"#).unwrap(), @"x space.nobreak negthinspace  thin  med  med  thick  thick  quad  wide  y "
    );
}

#[test]
fn test_convert_escape() {
    assert_snapshot!(convert_math(r#"$\|x\|| \& \# \% \$ y$"#).unwrap(), @"|| x || |  amp  hash  %  dollar  y "
    );
    assert_snapshot!(convert_math(r#"$a*b * c$"#).unwrap(), @r###"a \*b  \* c "###
    );
    assert_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###"\""###
    );
    assert_snapshot!(convert_text(r#"@abc"#).unwrap(), @r###"\@abc"###
    );
}

#[test]
fn test_unreachable() {
    // println!("{:#?}", convert_math(r#"$u^-$"#));
    assert_snapshot!(convert_math(r#"$u^−$"#).unwrap(), @"u ^(− )"
    );
}

#[test]
fn test_convert_sqrt() {
    assert_snapshot!(convert_math(r#"$\sqrt 1$"#).unwrap(), @"mitexsqrt(1 )");
    assert_snapshot!(convert_math(r#"$\sqrt [1]2$"#).unwrap(), @r###"mitexsqrt(\[1 \],2 )"###
    );
}

#[test]
fn test_convert_lr() {
    assert_snapshot!(convert_math(r#"$\left.\right.$"#).unwrap(), @"lr(  )");
    assert_snapshot!(convert_math(r#"$\left.a\right.$"#).unwrap(), @"lr( a  )");
    assert_snapshot!(convert_math(r#"$\alpha\left.\right.$"#).unwrap(), @"alpha lr(  )"
    );
    assert_snapshot!(convert_math(r#"$\left  . a \right    \|$"#).unwrap(), @"lr(     a       || )"
    );
    assert_snapshot!(convert_math(r#"$\left\langle a\right\|$"#).unwrap(), @"lr(angle.l  a || )"
    );
    assert_snapshot!(convert_math(r#"$\left\lbrack\lbrack x\rbrack\right\rbrack$"#).unwrap(), @"lr(bracket.l bracket.l  x bracket.r bracket.r )"
    );
}

#[test]
fn test_convert_color() {
    assert_snapshot!(convert_math(r#"$x\color{red}yz\frac{1}{2}$"#).unwrap(), @"x mitexcolor(r e d ,y z frac(1 ,2 ))");
    assert_snapshot!(convert_math(r#"$x\textcolor{red}yz$"#).unwrap(), @"x colortext(r e d ,y )z ");
    assert_snapshot!(convert_math(r#"$x\textcolor{red}{yz}$"#).unwrap(), @"x colortext(r e d ,y z )");
    assert_snapshot!(convert_math(r#"$x\colorbox{red}yz$"#).unwrap(), @"x colorbox(r e d ,y )z "
    );
    assert_snapshot!(convert_math(r#"$x\colorbox{red}{yz}$"#).unwrap(), @"x colorbox(r e d ,y z )"
    );
}

#[test]
fn test_convert_matrix() {
    assert_snapshot!(convert_math(
                 r#"$\begin{pmatrix}x{\\}x\end{pmatrix}$"#
        ).unwrap(),
        @"pmatrix(x x )"
    );
    assert_snapshot!(convert_math(
                 r#"$\begin{pmatrix} \\ & \ddots \end{pmatrix}$"#
        ).unwrap(),
        @"pmatrix( zws ; zws , dots.down  )"
    );
    assert_snapshot!(convert_math(
            r#"$\begin{matrix}
    1 & 2 & 3\\
a & b & c
\end{matrix}$"#
        ).unwrap(),
        @r###"
    matrix(
    1  zws , 2  zws , 3 zws ;
    a  zws , b  zws , c 
    )
    "###
    );
    assert_snapshot!(convert_math(
            r#"$\begin{Vmatrix}
    1 & 2 & 3\\
a & b & c
\end{Vmatrix}$"#
        ).unwrap(),
        @r###"
    Vmatrix(
    1  zws , 2  zws , 3 zws ;
    a  zws , b  zws , c 
    )
    "###
    );
    assert_snapshot!(convert_math(
            r#"$\begin{array}{lcr}
    1 & 2 & 3\\
a & b & c
\end{array}$"#
        ).unwrap(),
        @r###"
    mitexarray(arg0: l c r ,
    1  zws , 2  zws , 3 zws ;
    a  zws , b  zws , c 
    )
    "###
    );
}

#[test]
fn test_convert_env() {
    assert_snapshot!(convert_math(
            r#"$\begin{aligned}
    1 & 2 & 3\\
a & b & c
\end{aligned}$"#
        ).unwrap(),
        @r###"
    aligned(
    1  & 2  & 3 \ 
    a  & b  & c 
    )
    "###
    );
    assert_snapshot!(convert_math(
            r#"$\begin{align*}
    1 & 2 & 3\\
a & b & c
\end{align*}$"#
        ).unwrap(),
        @r###"
    aligned(
    1  & 2  & 3 \ 
    a  & b  & c 
    )
    "###
    );
    assert_snapshot!(convert_math(
            r#"$\begin{cases}
    1 & 2 & 3\\
a & b & c
\end{cases}$"#
        ).unwrap(),
        @r###"
    cases(
    1  & 2  & 3 ,
    a  & b  & c 
    )
    "###
    );
}

#[test]
fn test_convert_ditto() {
    assert_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###"\""###);
    assert_snapshot!(convert_math(r#"$a"b"c$"#).unwrap(), @r###"a \"b \"c "###);
    assert_snapshot!(convert_math(r#"$\text{a"b"c}$"#).unwrap(), @r###"#textmath[a\"b\"c];"###);
    assert_snapshot!(convert_math(r#"$\text{a " b " c}$"#).unwrap(), @r###"#textmath[a \" b \" c];"###);
}

#[test]
fn test_convert_text() {
    assert_snapshot!(convert_math(r#"$\text{abc}$"#).unwrap(), @"#textmath[abc];");
    assert_snapshot!(convert_math(r#"$\text{ a b c }$"#).unwrap(), @"#textmath[ a b c ];");
    assert_snapshot!(convert_math(r#"$\text{abc{}}$"#).unwrap(), @"#textmath[abc];");
    assert_snapshot!(convert_math(r#"$\text{ab{}c}$"#).unwrap(), @"#textmath[abc];");
    assert_snapshot!(convert_math(r#"$\text{ab c}$"#).unwrap(), @"#textmath[ab c];");
    assert_snapshot!(convert_math(r#"$\text{ab$x$c}$"#).unwrap(), @"#textmath[ab#math.equation(block: false, $x $);c];");
    assert_snapshot!(convert_math(r#"$\text{ab*c}$"#).unwrap(), @r###"#textmath[ab\*c];"###);
    assert_snapshot!(convert_math(r#"$\text{ab_c}$"#).unwrap(), @r###"#textmath[ab\_c];"###);
    assert_snapshot!(convert_math(r#"$\text{ab^c}$"#).unwrap(), @r###"#textmath[ab\^c];"###);
    // note: hack doesn't work in this case
    assert_snapshot!(convert_math(r#"$\text{ab\color{red}c}$"#).unwrap(), @"#textmath[abmitexcolor(red,c)];");
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
    assert_snapshot!(convert_text(r#"$a$"#).unwrap(), @"#math.equation(block: false, $a $);");
    assert_snapshot!(convert_text(r#"$$a$$"#).unwrap(), @"$ a  $");
    assert_snapshot!(convert_text(r#"\(a\)"#).unwrap(), @"#math.equation(block: false, $a $);");
    assert_snapshot!(convert_text(r#"\[a\]"#).unwrap(), @"$ a  $");
    assert_snapshot!(convert_text(r#"$ a $"#).unwrap(), @"#math.equation(block: false, $ a  $);");
    assert_snapshot!(convert_text(r#"$$ a $ $ b $$"#).unwrap_err(), @"error: formula is not valid");
    assert_snapshot!(convert_text(r#"\[a\)\(b\]"#).unwrap_err(), @"error: formula is not valid");
}

#[test]
fn test_fuzzing() {
    assert_snapshot!(convert_math(r#"\left\0"#).unwrap_err(), @r###"error: unknown command: \0"###);
    assert_snapshot!(convert_math(r#"\end{}"#).unwrap_err(), @r###"error: error unexpected: """###);
}
