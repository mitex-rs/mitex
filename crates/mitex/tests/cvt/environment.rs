use super::prelude::*;

#[test]
fn easy() {
    assert_snapshot!(convert_text(r#"\begin{equation}\end{equation}"#).unwrap(), @"$ aligned() $");
    assert_snapshot!(convert_math(r#"\begin{equation}\end{equation}"#).unwrap(), @"aligned()");
}

#[test]
fn matrix() {
    assert_snapshot!(convert_math(
            r#"\begin{matrix}
a & b \\
c & d
\end{matrix}"#).unwrap(), @r###"
    matrix(
    a  zws , b  zws ;
    c  zws , d 
    )
    "###);
    assert_snapshot!(convert_math(
            r#"\begin{pmatrix}\\\end{pmatrix}"#).unwrap(), @"pmatrix(zws ;)");
    assert_snapshot!(convert_math(
            r#"\begin{pmatrix}x{\\}x\end{pmatrix}"#).unwrap(), @"pmatrix(x x )");
}

#[test]
fn arguments() {
    assert_snapshot!(convert_math(
            r#"\begin{array}{lc}
a & b \\
c & d
\end{array}"#).unwrap(), @r###"
    mitexarray(arg0: l c ,
    a  zws , b  zws ;
    c  zws , d 
    )
    "###);
}

#[test]
fn space_around_and() {
    assert_snapshot!(convert_math(
            r#"\begin{bmatrix}A&B\end{bmatrix}"#).unwrap(), @"bmatrix(A zws ,B )");
}
