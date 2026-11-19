use super::prelude::*;
use mitex::command_preludes::{define_command_with_alias, define_symbol, SpecBuilder};

/// The default specification extended with a few user-defined commands.
fn spec_with_custom_commands() -> mitex::CommandSpec {
    let mut builder = SpecBuilder::default();
    for (name, item) in DEFAULT_SPEC.items() {
        builder.add_command(name, item.clone());
    }
    builder.add_command("mycmd", define_command_with_alias(1, "mycmd"));
    builder.add_command("mysym", define_symbol("alpha"));
    builder.build()
}

#[test]
fn test_custom_command_is_converted() {
    // Description: A command added to a custom specification must be known not
    // only to the parser but also to the converter. Previously the converter
    // always used the default specification and failed with
    // "unknown command: \mycmd".
    let spec = spec_with_custom_commands();
    assert_snapshot!(mitex_convert_math(r#"\mycmd{x} + y"#, Some(spec.clone())).unwrap(), @"mycmd(x ) +  y ");
    assert_snapshot!(mitex_convert_math(r#"\mysym"#, Some(spec)).unwrap(), @"alpha ");
}

#[test]
fn test_custom_command_in_text_mode() {
    let spec = spec_with_custom_commands();
    assert_snapshot!(mitex_convert_text(r#"$\mycmd{x}$"#, Some(spec)).unwrap(), @"#math.equation(block: false, $mycmd(x )$);");
}
