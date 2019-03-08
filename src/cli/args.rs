use clap::{App, Shell};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "minittc",
    rename_all = "kebab-case",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
pub struct CliOptions {
    /// Parses but do not type-check the input file
    #[structopt(short = "p", long)]
    pub parse_only: bool,
    /// Prints code generated from parsed AST (in most cases it's accepted by the parser as well)
    #[structopt(short = "g", long)]
    pub generated: bool,
    /// Interactive mode, aka REPL
    #[structopt(alias = "repl", short = "i", long)]
    pub interactive: bool,
    /// Interactive mode without completion/hints/colored output
    #[structopt(alias = "repl-plain", short = "j", long)]
    pub interactive_plain: bool,
    /// Prints errors only
    #[structopt(short = "q", long)]
    pub quiet: bool,
    /// the input file to type-check (Notice: file should be UTF-8 encoded)
    #[structopt(name = "FILE")]
    pub file: Option<String>,
    #[structopt(subcommand)]
    completion: Option<GenShellSubCommand>,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum GenShellSubCommand {
    /// Prints completion scripts for your shell
    Completion {
        /// Prints completion scripts for your shell
        #[structopt(
            name = "generate-completion-script-for",
            alias = "gcf",
            raw(possible_values = "&Shell::variants()", case_insensitive = "true")
        )]
        shell: Shell,
    },
}

fn app<'a, 'b>() -> App<'a, 'b> {
    let extra_help = "For extra help please head to \
                      https://github.com/owo-lang/minitt-rs/issues/new";
    // Introduced a variable because stupid CLion :(
    let app: App = CliOptions::clap();
    app.after_help(extra_help)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
}

pub fn pre() -> CliOptions {
    let args: CliOptions = CliOptions::from_clap(&app().get_matches());
    if let Some(GenShellSubCommand::Completion { shell }) = args.completion {
        app().gen_completions_to("minittc", shell, &mut std::io::stdout());
    }
    args
}
