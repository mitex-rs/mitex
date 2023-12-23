pub mod command {
    use crate::{ArgShape, CommandSpecItem};

    pub fn define_command(num: u8) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(num)),
            alias: None,
        })
    }

    pub fn define_glob_command(reg: &str, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::Glob(reg.into())),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_symbol(alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::None),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_command_with_alias(num: u8, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(num)),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_greedy_command(alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::Greedy),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_matrix_env(num: Option<u8>, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Env(crate::EnvShape {
            args: num
                .map(crate::ArgPattern::FixedLenTerm)
                .unwrap_or(crate::ArgPattern::None),
            ctx_feature: crate::ContextFeature::IsMatrix,
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_normal_env(num: Option<u8>, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Env(crate::EnvShape {
            args: num
                .map(crate::ArgPattern::FixedLenTerm)
                .unwrap_or(crate::ArgPattern::None),
            ctx_feature: crate::ContextFeature::None,
            alias: Some(alias.to_owned()),
        })
    }
    pub const fn define_const_command(args: ArgShape) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape { args, alias: None })
    }

    pub const TEX_CMD0: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(0)));
    pub const TEX_CMD1: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(1)));
    pub const TEX_CMD2: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(2)));
    pub const TEX_SYMBOL: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::None));
    pub const TEX_LEFT1_OPEARTOR: CommandSpecItem = define_const_command(crate::ArgShape::Left1);
    pub const TEX_GREEDY_OPERATOR: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::Greedy));
    pub const TEX_INFIX_OPERATOR: CommandSpecItem =
        define_const_command(crate::ArgShape::InfixGreedy);
    pub const TEX_MATRIX_ENV: CommandSpecItem = CommandSpecItem::Env(crate::EnvShape {
        args: crate::ArgPattern::None,
        ctx_feature: crate::ContextFeature::IsMatrix,
        alias: None,
    });
    pub const TEX_NORMAL_ENV: CommandSpecItem = CommandSpecItem::Env(crate::EnvShape {
        args: crate::ArgPattern::None,
        ctx_feature: crate::ContextFeature::None,
        alias: None,
    });

    #[derive(Default)]
    pub struct SpecBuilder {
        commands: fxhash::FxHashMap<String, CommandSpecItem>,
    }

    impl SpecBuilder {
        pub fn add_command(&mut self, name: &str, item: CommandSpecItem) -> &mut Self {
            self.commands.insert(name.to_owned(), item);
            self
        }

        pub fn build(self) -> crate::CommandSpec {
            crate::CommandSpec::new(self.commands)
        }
    }
}
