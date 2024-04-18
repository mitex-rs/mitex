use super::prelude::*;

#[test]
fn test_starrd_command() {
    // Description: If the starred command is defined, it is treated as a starred
    assert_snapshot!(convert_math(r#"\operatorname*{a}"#).unwrap(), @"operatornamewithlimits(a )"
    );
    // Description: If the starred command is not defined, it is treated as a normal
    // command
    assert_snapshot!(convert_math(r#"\varphi*1"#).unwrap(), @r###"phi \*1 "###
    );
}

#[test]
fn left_association() {
    assert_snapshot!(convert_math(r#"\sum"#).unwrap(), @"sum ");
    assert_snapshot!(convert_math(r#"\sum\limits"#).unwrap(), @"limits(sum )");
    assert_snapshot!(convert_math(r#"\sum \limits"#).unwrap(), @"limits(sum  )");
    assert_snapshot!(convert_math(r#"\sum\limits\limits"#).unwrap(), @"limits(limits(sum ))");
    assert_snapshot!(convert_math(r#"\sum\limits\sum"#).unwrap(), @"limits(sum )sum ");
    assert_snapshot!(convert_math(r#"\sum\limits\sum\limits"#).unwrap(), @"limits(sum )limits(sum )");
    assert_snapshot!(convert_math(r#"\limits"#).unwrap(), @"limits()");
}

#[test]
fn greedy_assosiation() {
    // Description: infix greed and right greedy
    assert_snapshot!(convert_math(r#"1 \over \displaystyle 2"#).unwrap(), @"frac(1  , mitexdisplay( 2 ))");
    // Description: right greed and right greedy
    assert_snapshot!(convert_math(r#"\displaystyle \displaystyle 1 \over 2"#).unwrap(), @"mitexdisplay( mitexdisplay(frac( 1  , 2 )))");
    // Description: right greed and infix greedy
    assert_snapshot!(convert_math(r#"\displaystyle 1 \over 2"#).unwrap(), @"mitexdisplay(frac( 1  , 2 ))");
    // Description: infix greed and infix greedy
    // Note: this is an invalid expression
    assert_snapshot!(convert_math(r#"a \over c \over b"#).unwrap(), @"frac(a  ,frac( c  , b ))");
}

#[test]
fn right_greedy() {
    // Description: produces an empty argument if the righ side is empty
    assert_snapshot!(convert_math(r#"\displaystyle"#).unwrap(), @"mitexdisplay()");
    // Description: correctly works left association
    // left1 commands
    assert_snapshot!(convert_math(r#"\displaystyle\sum\limits"#).unwrap(), @"mitexdisplay(limits(sum ))");
    // subscript
    assert_snapshot!(convert_math(r#"\displaystyle x_1"#).unwrap(), @"mitexdisplay( x _(1 ))");
    // prime
    assert_snapshot!(convert_math(r#"\displaystyle x'"#).unwrap(), @"mitexdisplay( x ')");
    // Description: doesn't panic on incorect left association
    // left1 commands
    assert_snapshot!(convert_math(r#"\displaystyle\limits"#).unwrap(), @"mitexdisplay(limits())");
    // subscript
    assert_snapshot!(convert_math(r#"\displaystyle_1"#).unwrap(), @"mitexdisplay(zws_(1 ))");
    // prime
    assert_snapshot!(convert_math(r#"\displaystyle'"#).unwrap(), @"mitexdisplay(')");
    // Description: all right side content is collected to a single argument
    assert_snapshot!(convert_math(r#"\displaystyle a b c"#).unwrap(), @"mitexdisplay( a  b  c )");
    assert_snapshot!(convert_math(r#"\displaystyle \sum T"#).unwrap(), @"mitexdisplay( sum  T )");
    // Curly braces doesn't start a new argument
    assert_snapshot!(convert_math(r#"\displaystyle{\sum T}"#).unwrap(), @"mitexdisplay(sum  T )");
    // Description: doesn't identify brackets as group
    assert_snapshot!(convert_math(r#"\displaystyle[\sum T]"#).unwrap(), @r###"mitexdisplay(\[sum  T \])"###);
    // Description: scoped by curly braces
    assert_snapshot!(convert_math(r#"a + {\displaystyle a b} c"#).unwrap(), @"a  +  mitexdisplay( a  b ) c ");
    // Description: doeesn't affect left side
    assert_snapshot!(convert_math(r#"T \displaystyle"#).unwrap(), @"T  mitexdisplay()");
}

#[test]
fn infix() {
    assert_snapshot!(convert_math(r#"\over_1"#).unwrap(), @"frac(,zws_(1 ))");
    assert_snapshot!(convert_math(r#"\over'"#).unwrap(), @"frac(,')");
    assert_snapshot!(convert_math(r#"a \over b'_1"#).unwrap(), @"frac(a  , b '_(1 ))");
    assert_snapshot!(convert_math(r#"a \over b"#).unwrap(), @"frac(a  , b )");
    assert_snapshot!(convert_math(r#"1 + {2 \over 3}"#).unwrap(), @"1  +  frac(2  , 3 )");
}
