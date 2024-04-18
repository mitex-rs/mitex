use super::prelude::*;

/// Argument will reset flag of being in a formula
#[test]
fn arg_scope() {
    assert_snapshot!(convert_math(r#"$\text{${1}$}$"#).unwrap(), @"#textmath[#math.equation(block: false, $1 $);];");
    // Note: This is a valid AST, but semantically incorrect (indicated by overleaf)
    assert_snapshot!(convert_math(r#"$\frac{${1}$}{${2}$}$"#).unwrap(), @"frac(1 ,2 )");
}
