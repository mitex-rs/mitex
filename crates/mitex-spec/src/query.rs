use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

/// A package specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    /// The name of the package.
    pub name: String,
    /// The command specification of the package.
    pub spec: CommandSpecRepr,
}

/// A ordered list of package specifications.
///
/// The latter package specification will override the former one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesVec(pub Vec<PackageSpec>);

/// An item of command specification.
/// This enum contains more sugar than the canonical representation.
///
/// See [`crate::CommandSpecItem`] for more details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CommandSpecItem {
    #[serde(rename = "cmd")]
    Cmd(CmdShape),
    #[serde(rename = "env")]
    Env(EnvShape),
    /// A command that takes no argument, and its handler is also a typst
    /// symbol.
    #[serde(rename = "sym")]
    Symbol,
    /// A command that takes zero argument, and its handler is a typst function.
    #[serde(rename = "cmd0")]
    Command0,
    /// A command that takes one argument.
    #[serde(rename = "cmd1")]
    Command1,
    /// A command that takes two arguments.
    #[serde(rename = "cmd2")]
    Command2,
    /// A command that takes one argument and is a left-associative operator.
    #[serde(rename = "left1-cmd")]
    CmdLeft1,
    /// A command that takes no argument and is a matrix environment.
    #[serde(rename = "matrix-env")]
    EnvMatrix,
    /// A command that takes no argument and is a normal environment.
    #[serde(rename = "normal-env")]
    EnvNormal,

    /// A command that is aliased to a Typst symbol.
    #[serde(rename = "alias-sym")]
    SymAlias { alias: String },
    /// A command that is greedy and is aliased to a Typst handler.
    #[serde(rename = "greedy-cmd")]
    CmdGreedy { alias: String },
    #[serde(rename = "infix-cmd")]
    /// A command that is an infix operator and is aliased to a Typst handler.
    CmdInfix { alias: String },
    #[serde(rename = "glob-cmd")]
    /// A command that has a glob argument pattern and is aliased to a Typst
    /// handler.
    CmdGlob { pattern: String, alias: String },
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
            CommandSpecItem::CmdLeft1 => TEX_LEFT1_OPEARTOR,
            CommandSpecItem::EnvMatrix => TEX_MATRIX_ENV,
            CommandSpecItem::EnvNormal => TEX_NORMAL_ENV,
            CommandSpecItem::SymAlias { alias } => define_symbol(&alias),
            CommandSpecItem::CmdGreedy { alias } => define_greedy_command(&alias),
            CommandSpecItem::CmdInfix { alias } => crate::CommandSpecItem::Cmd(crate::CmdShape {
                args: crate::ArgShape::InfixGreedy,
                alias: Some(alias.to_owned()),
            }),
            CommandSpecItem::CmdGlob { pattern, alias } => define_glob_command(&pattern, &alias),
        }
    }
}

/// The following defined structs are copied so we don't maintain their
/// comments. See [`crate::CommandSpecRepr`] for canonical representation.
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ArgPattern {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "fixed-len")]
    FixedLenTerm { len: u8 },
    #[serde(rename = "range-len")]
    RangeLenTerm { min: u8, max: u8 },
    #[serde(rename = "greedy")]
    Greedy,
    #[serde(rename = "glob")]
    Glob { pattern: Box<str> },
}

impl From<ArgPattern> for crate::ArgPattern {
    fn from(pattern: ArgPattern) -> Self {
        match pattern {
            ArgPattern::None => Self::None,
            ArgPattern::FixedLenTerm { len } => Self::FixedLenTerm(len),
            ArgPattern::RangeLenTerm { min, max } => Self::RangeLenTerm(min, max),
            ArgPattern::Greedy => Self::Greedy,
            ArgPattern::Glob { pattern } => Self::Glob(pattern.into()),
        }
    }
}

// struct ArgShape(ArgPattern, Direction);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ArgShape {
    #[serde(rename = "right")]
    Right { pattern: ArgPattern },
    #[serde(rename = "left1")]
    Left1,
    #[serde(rename = "infix-greedy")]
    InfixGreedy,
}

impl From<ArgShape> for crate::ArgShape {
    fn from(shape: ArgShape) -> Self {
        match shape {
            ArgShape::Right { pattern } => Self::Right(pattern.into()),
            ArgShape::Left1 => Self::Left1,
            ArgShape::InfixGreedy => Self::InfixGreedy,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ContextFeature {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "is-matrix")]
    IsMatrix,
    #[serde(rename = "is-cases")]
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
