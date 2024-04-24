//! The query module contains the data structures that are used by `typst query
//! <mitex-packages>`

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{CmdShape, ContextFeature, EnvShape};

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
    /// A canonical command item.
    #[serde(rename = "cmd")]
    Cmd(CmdShape),
    /// A canonical environment item.
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
    /// A command that has a glob argument pattern and is an environment.
    #[serde(rename = "glob-env")]
    EnvGlob {
        /// The glob pattern of the command.
        pattern: String,
        /// The aliasing typst handle of the command.
        alias: String,
        /// The context feature of the command.
        ctx_feature: ContextFeature,
    },

    /// A command that is aliased to a Typst symbol.
    #[serde(rename = "alias-sym")]
    SymAlias {
        /// The aliasing typst handle of the symbol.
        alias: String,
    },
    /// A command that is greedy and is aliased to a Typst handler.
    #[serde(rename = "greedy-cmd")]
    CmdGreedy {
        /// The aliasing typst handle of the command.
        alias: String,
    },
    /// A command that is an infix operator and is aliased to a Typst handler.
    #[serde(rename = "infix-cmd")]
    CmdInfix {
        /// The aliasing typst handle of the command.
        alias: String,
    },
    /// A command that has a glob argument pattern and is aliased to a Typst
    /// handler.
    #[serde(rename = "glob-cmd")]
    CmdGlob {
        /// The glob pattern of the command.
        pattern: String,
        /// The aliasing typst handle of the command.
        alias: String,
    },
}

impl From<CommandSpecItem> for crate::CommandSpecItem {
    fn from(item: CommandSpecItem) -> Self {
        use crate::preludes::command::*;
        match item {
            CommandSpecItem::Cmd(shape) => Self::Cmd(shape),
            CommandSpecItem::Env(shape) => Self::Env(shape),
            CommandSpecItem::Symbol => TEX_SYMBOL,
            CommandSpecItem::Command0 => TEX_CMD0,
            CommandSpecItem::Command1 => TEX_CMD1,
            CommandSpecItem::Command2 => TEX_CMD2,
            CommandSpecItem::CmdLeft1 => TEX_LEFT1_OPEARTOR,
            CommandSpecItem::EnvMatrix => TEX_MATRIX_ENV,
            CommandSpecItem::EnvNormal => TEX_NORMAL_ENV,
            CommandSpecItem::EnvGlob {
                pattern,
                alias,
                ctx_feature,
            } => define_glob_env(&pattern, &alias, ctx_feature),
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

/// Command specification that contains a set of commands and environments. It
/// is used for us to define the meta data of LaTeX packages in typst code and
/// query by `typst query` then. See [`Spec`] for an example.
///
/// Note: There are non-canonical format of items could be used for convenience.
///
/// [`Spec`]: https://github.com/mitex-rs/mitex/tree/main/packages/mitex/specs
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpecRepr {
    /// The command specifications.
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
