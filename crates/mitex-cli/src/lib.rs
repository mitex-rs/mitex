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
    /// Print Version
    #[arg(short = 'V', long, group = "version-dump")]
    pub version: bool,

    /// Print Version in format
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
    /// Generate an AST.
    Syntax,
    /// Generate an LaTeX document.
    All,
}

/// Compile arguments.
#[derive(Default, Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Path to typst workspace.
    #[clap(long, short, default_value = ".")]
    pub workspace: String,

    /// Entry file.
    #[clap(long, short, default_value = "")]
    pub input: String,

    /// Compile stage.
    #[clap(long, short, value_enum)]
    pub stage: Option<CompileStage>,

    /// Output to file, default to entry file name with `.tex` extension.
    #[clap(long, short, default_value = "")]
    pub output: String,

    /// Default positional arguments for input and output file.
    #[arg(trailing_var_arg = true, hide = true)]
    _i_or_o_args: Vec<String>,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "The CLI for MiTeX.",
    after_help = "",
    next_display_order = None
)]

/// Subcommands
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    /// Run MiTeX transpiler.
    #[clap(visible_alias = "c", about = "Run compiler.")]
    Compile(CompileArgs),

    /// Generate shell completion script.
    #[clap(about = "Generate shell completion script.")]
    Completion(CompletionArgs),

    /// Generate manual.
    #[clap(about = "Generate manual.")]
    Manual(ManualArgs),

    /// Subcommands about spec for MiTeX.
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
    /// Path to output directory
    pub dest: PathBuf,
}

/// Commands about spec for MiTeX.
#[derive(Debug, Subcommand)]
#[clap(
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum SpecSubCommands {
    /// Generate Specification
    Generate(GenerateSpecArgs),
}

/// Generate specification for MiTeX.
#[derive(Debug, Clone, Parser)]
pub struct GenerateSpecArgs {}

/// Struct for build info.
pub mod build_info {
    /// The version of the mitex-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Get CLI command instance with/without subcommand.
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
                    args.input = io_args[0].clone();
                }
                2 => {
                    args.input = io_args[0].clone();
                    args.output = io_args[1].clone();
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
            args.output = std::path::Path::new(&args.input)
                .with_extension("tex")
                .to_str()
                .ok_or_else(|| {
                    clap::Error::raw(
                        clap::error::ErrorKind::ValueValidation,
                        "Input file name is invalid.",
                    )
                })?
                .to_owned();
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
