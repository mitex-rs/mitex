use super::prelude::*;

#[test]
fn test_starrd_command() {
    // Description: If the starred command is defined, it is treated as a starred
    assert_debug_snapshot!(convert_math(r#"\operatorname*{a}"#), @r###"
    Ok(
        "operatornamewithlimits(a )",
    )
    "###
    );
    // Description: If the starred command is not defined, it is treated as a normal
    // command
    assert_debug_snapshot!(convert_math(r#"\varphi*1"#), @r###"
    Ok(
        "phi \\*1 ",
    )
    "###
    );
}

#[test]
fn left_association() {
    assert_debug_snapshot!(convert_math(r#"\sum"#), @r###"
    Ok(
        "sum ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sum\limits"#), @r###"
    Ok(
        "limits(sum )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sum \limits"#), @r###"
    Ok(
        "limits(sum  )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sum\limits\limits"#), @r###"
    Ok(
        "limits(limits(sum ))",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sum\limits\sum"#), @r###"
    Ok(
        "limits(sum )sum ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\sum\limits\sum\limits"#), @r###"
    Ok(
        "limits(sum )limits(sum )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\limits"#), @r###"
    Ok(
        "limits()",
    )
    "###);
}

#[test]
fn greedy_assosiation() {
    // Description: infix greed and right greedy
    assert_debug_snapshot!(convert_math(r#"1 \over \displaystyle 2"#), @r###"
    Ok(
        "frac(1  , mitexdisplay( 2 ))",
    )
    "###);
    // Description: right greed and right greedy
    assert_debug_snapshot!(convert_math(r#"\displaystyle \displaystyle 1 \over 2"#), @r###"
    Ok(
        "mitexdisplay( mitexdisplay(frac( 1  , 2 )))",
    )
    "###);
    // Description: right greed and infix greedy
    assert_debug_snapshot!(convert_math(r#"\displaystyle 1 \over 2"#), @r###"
    Ok(
        "mitexdisplay(frac( 1  , 2 ))",
    )
    "###);
    // Description: infix greed and infix greedy
    // Note: this is an invalid expression
    assert_debug_snapshot!(convert_math(r#"a \over c \over b"#), @r###"
    Ok(
        "frac(a  ,frac( c  , b ))",
    )
    "###);
}

#[test]
fn right_greedy() {
    // Description: produces an empty argument if the righ side is empty
    assert_debug_snapshot!(convert_math(r#"\displaystyle"#), @r###"
    Ok(
        "mitexdisplay()",
    )
    "###);
    // Description: correctly works left association
    // left1 commands
    assert_debug_snapshot!(convert_math(r#"\displaystyle\sum\limits"#), @r###"
    Ok(
        "mitexdisplay(limits(sum ))",
    )
    "###);
    // subscript
    assert_debug_snapshot!(convert_math(r#"\displaystyle x_1"#), @r###"
    Ok(
        "mitexdisplay( x _(1 ))",
    )
    "###);
    // prime
    assert_debug_snapshot!(convert_math(r#"\displaystyle x'"#), @r###"
    Ok(
        "mitexdisplay( x ')",
    )
    "###);
    // Description: doesn't panic on incorect left association
    // left1 commands
    assert_debug_snapshot!(convert_math(r#"\displaystyle\limits"#), @r###"
    Ok(
        "mitexdisplay(limits())",
    )
    "###);
    // subscript
    assert_debug_snapshot!(convert_math(r#"\displaystyle_1"#), @r###"
    Ok(
        "mitexdisplay(zws_(1 ))",
    )
    "###);
    // prime
    assert_debug_snapshot!(convert_math(r#"\displaystyle'"#), @r###"
    Ok(
        "mitexdisplay(')",
    )
    "###);
    // Description: all right side content is collected to a single argument
    assert_debug_snapshot!(convert_math(r#"\displaystyle a b c"#), @r###"
    Ok(
        "mitexdisplay( a  b  c )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\displaystyle \sum T"#), @r###"
    Ok(
        "mitexdisplay( sum  T )",
    )
    "###);
    // Curly braces doesn't start a new argument
    assert_debug_snapshot!(convert_math(r#"\displaystyle{\sum T}"#), @r###"
    Ok(
        "mitexdisplay(sum  T )",
    )
    "###);
    // Description: doesn't identify brackets as group
    assert_debug_snapshot!(convert_math(r#"\displaystyle[\sum T]"#), @r###"
    Ok(
        "mitexdisplay(\\[sum  T \\])",
    )
    "###);
    // Description: scoped by curly braces
    assert_debug_snapshot!(convert_math(r#"a + {\displaystyle a b} c"#), @r###"
    Ok(
        "a  +  mitexdisplay( a  b ) c ",
    )
    "###);
    // Description: doeesn't affect left side
    assert_debug_snapshot!(convert_math(r#"T \displaystyle"#), @r###"
    Ok(
        "T  mitexdisplay()",
    )
    "###);
}

#[test]
fn infix() {
    assert_debug_snapshot!(convert_math(r#"\over_1"#), @r###"
    Ok(
        "frac(,zws_(1 ))",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\over'"#), @r###"
    Ok(
        "frac(,')",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"a \over b'_1"#), @r###"
    Ok(
        "frac(a  , b '_(1 ))",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"a \over b"#), @r###"
    Ok(
        "frac(a  , b )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"1 + {2 \over 3}"#), @r###"
    Ok(
        "1  +  frac(2  , 3 )",
    )
    "###);
}
