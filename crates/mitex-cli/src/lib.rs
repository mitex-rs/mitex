//! The CLI for MiTeX.

pub mod utils;
mod version;

pub use version::{intercept_version, VersionFormat};

use std::path::PathBuf;

use build_info::VERSION;
use clap::{Args, Command, FromArgMatches, Parser, Subcommand, ValueEnum};

/// CLI options
#[derive(Debug, Parser)]
#[clap(name = "mitex-cli", version = VERSION)]
pub struct Opts {
    /// Prints version
    #[arg(short = 'V', long, group = "version-dump")]
    pub version: bool,

    /// Prints version in format
    #[arg(long = "VV", alias = "version-fmt", group = "version-dump", default_value_t = VersionFormat::None)]
    pub vv: VersionFormat,

    /// Subcommands
    #[clap(subcommand)]
    pub sub: Option<Subcommands>,
}

/// Available compile stages for `$program compile`
#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
#[value(rename_all = "kebab-case")]
pub enum CompileStage {
    /// Generates an AST.
    Syntax,
    /// Generates a Typst document.
    Document,
}

/// Compile arguments.
#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Path to workspace.
    ///
    /// This is used to resolve imports in `\iftypst` blocks.
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile -w '/my-workspace' main.tex
    /// ```
    ///
    /// Resolves `/some-file.typ` with `/my-workspace/some-file.typ`
    ///
    /// ```latex
    /// \iftypst
    /// #import "/some-file.typ"
    /// \fi
    /// ```
    #[clap(long, short, default_value = ".")]
    pub workspace: String,

    /// Entry file.
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile main.tex
    /// ```
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile -i main.tex
    /// ```
    #[clap(long, short, default_value = "")]
    pub input: String,

    /// Compile stage.
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile --stage syntax main.tex
    /// ```
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile --stage document main.tex
    /// ```
    #[clap(long, value_enum)]
    pub stage: Option<CompileStage>,

    /// Output to file, default to entry file name with `.typ` extension.
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile main.tex main.typ
    /// ```
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile -i main.tex -o main.typ
    /// ```
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile --stage syntax main.tex main.ast.txt
    /// ```
    #[clap(long, short, default_value = "")]
    pub output: String,

    /// Default positional arguments for input and output file.
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile main.tex
    /// ```
    ///
    /// ## Example
    ///
    /// ```bash
    /// mitex compile main.tex main.typ
    /// ```
    #[arg(trailing_var_arg = true, hide = true)]
    _i_or_o_args: Vec<String>,
}

/// Subcommands
#[derive(Debug, Subcommand)]
#[clap(about = "The CLI for MiTeX.", next_display_order = None)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    /// Runs MiTeX transpiler.
    #[clap(visible_alias = "c")]
    Compile(CompileArgs),

    /// Generates a shell completion script.
    Completion(CompletionArgs),

    /// Generates a manual.
    Manual(ManualArgs),

    /// Subcommands about command specification for MiTeX.
    #[clap(subcommand)]
    Spec(SpecSubCommands),
}

/// Generate shell completion script.
#[derive(Debug, Clone, Parser)]
pub struct CompletionArgs {
    /// Completion script kind.
    #[clap(value_enum)]
    pub shell: clap_complete::Shell,
}

/// Generate shell completion script.
#[derive(Debug, Clone, Parser)]
pub struct ManualArgs {
    /// Path to output directory.
    pub dest: PathBuf,
}

/// Commands about command specification for MiTeX.
#[derive(Debug, Subcommand)]
#[clap(next_display_order = None)]
#[allow(clippy::large_enum_variant)]
pub enum SpecSubCommands {
    /// Generates a command specification file for MiTeX.
    Generate(GenerateSpecArgs),
}

/// Generates a command specification file for MiTeX.
#[derive(Debug, Clone, Parser)]
pub struct GenerateSpecArgs {}

/// Struct for build info.
pub mod build_info {
    /// The version of the mitex-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Get a CLI command instance with/without subcommand.
pub fn get_cli(sub_command_required: bool) -> Command {
    let cli = Command::new("$").disable_version_flag(true);
    Opts::augment_args(cli).subcommand_required(sub_command_required)
}

/// Process CLI options uniformly.
fn process_opts(mut opts: Opts) -> Result<Opts, clap::Error> {
    if let Some(Subcommands::Compile(args)) = &mut opts.sub {
        let io_args = std::mem::take(&mut args._i_or_o_args);
        if args.input.is_empty() && args.output.is_empty() {
            match io_args.len() {
                0 => {}
                1 => {
                    args.input.clone_from(&io_args[0]);
                }
                2 => {
                    args.input.clone_from(&io_args[0]);
                    args.output.clone_from(&io_args[1]);
                }
                _ => Err(clap::Error::raw(
                    clap::error::ErrorKind::ValueValidation,
                    "Too many positional arguments.",
                ))?,
            }
        } else if !io_args.is_empty() {
            Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                "Input and output file cannot be positional arguments\
                if any of them is specified by named argument.",
            ))?;
        }

        if args.input.is_empty() {
            Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                "Input file is required.",
            ))?;
        }
        if args.output.is_empty() {
            std::path::Path::new(&args.input)
                .with_extension("tex")
                .to_str()
                .ok_or_else(|| {
                    clap::Error::raw(
                        clap::error::ErrorKind::ValueValidation,
                        "Input file name is invalid.",
                    )
                })?
                .clone_into(&mut args.output);
        }
    }

    Ok(opts)
}

/// Get CLI options from command line arguments.
///
/// # Errors
/// Errors if the command line arguments are invalid.
pub fn get_os_opts(sub_command_required: bool) -> Result<Opts, clap::Error> {
    let res = Opts::from_arg_matches(&get_cli(sub_command_required).get_matches())?;

    process_opts(res)
}
