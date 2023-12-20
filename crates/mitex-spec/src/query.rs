use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

/// An item of command specification.
/// This contains more sugar than the canonical representation.
///
/// See [`mitex_spec::CommandSpecItem`] for more details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandSpecItem {
    Cmd(CmdShape),
    Env(EnvShape),
    /// A command that takes no argument, and its handler is also a typst
    /// symbol.
    Symbol,
    /// A command that takes zero argument, and its handler is a typst function.
    Command0,
    /// A command that takes one argument.
    Command1,
    /// A command that takes two arguments.
    Command2,
}

impl From<CommandSpecItem> for crate::CommandSpecItem {
    fn from(item: CommandSpecItem) -> Self {
        use crate::preludes::command::*;
        match item {
            CommandSpecItem::Cmd(shape) => Self::Cmd(shape.into()),
            CommandSpecItem::Env(shape) => Self::Env(shape.into()),
            CommandSpecItem::Symbol => TEX_SYMBOL,
            CommandSpecItem::Command0 => TEX_CMD0,
            CommandSpecItem::Command1 => TEX_CMD1,
            CommandSpecItem::Command2 => TEX_CMD2,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpecRepr {
    pub commands: HashMap<String, CommandSpecItem>,
}

impl From<CommandSpecRepr> for crate::CommandSpec {
    fn from(repr: CommandSpecRepr) -> Self {
        Self(Arc::new(repr.into()))
    }
}

impl From<CommandSpecRepr> for crate::CommandSpecRepr {
    fn from(repr: CommandSpecRepr) -> Self {
        Self {
            commands: repr
                .commands
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdShape {
    pub args: ArgShape,
    pub alias: Option<String>,
}

impl From<CmdShape> for crate::CmdShape {
    fn from(shape: CmdShape) -> Self {
        Self {
            args: shape.args.into(),
            alias: shape.alias,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvShape {
    pub args: ArgPattern,
    pub ctx_feature: ContextFeature,
    pub alias: Option<String>,
}

impl From<EnvShape> for crate::EnvShape {
    fn from(shape: EnvShape) -> Self {
        Self {
            args: shape.args.into(),
            ctx_feature: shape.ctx_feature.into(),
            alias: shape.alias,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgPattern {
    None,
    FixedLenTerm(u8),
    RangeLenTerm(u8, u8),
    Greedy,
    Glob(Box<str>),
}

impl From<ArgPattern> for crate::ArgPattern {
    fn from(pattern: ArgPattern) -> Self {
        match pattern {
            ArgPattern::None => Self::None,
            ArgPattern::FixedLenTerm(len) => Self::FixedLenTerm(len),
            ArgPattern::RangeLenTerm(min, max) => Self::RangeLenTerm(min, max),
            ArgPattern::Greedy => Self::Greedy,
            ArgPattern::Glob(glob) => Self::Glob(glob.into()),
        }
    }
}

// struct ArgShape(ArgPattern, Direction);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgShape {
    Right(ArgPattern),
    Left1,
    InfixGreedy,
}

impl From<ArgShape> for crate::ArgShape {
    fn from(shape: ArgShape) -> Self {
        match shape {
            ArgShape::Right(pattern) => Self::Right(pattern.into()),
            ArgShape::Left1 => Self::Left1,
            ArgShape::InfixGreedy => Self::InfixGreedy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextFeature {
    None,
    IsMatrix,
    IsCases,
}

impl From<ContextFeature> for crate::ContextFeature {
    fn from(feature: ContextFeature) -> Self {
        match feature {
            ContextFeature::None => Self::None,
            ContextFeature::IsMatrix => Self::IsMatrix,
            ContextFeature::IsCases => Self::IsCases,
        }
    }
}
