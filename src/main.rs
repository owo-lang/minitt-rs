use clap::{App, Shell};
use minitt::parser::parse_str_err_printed;
use minitt::type_check::check_main;
use std::io::Read;
use std::{fs, io, str};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "minittc",
    rename_all = "kebab-case",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
struct CliOptions {
    /// Parses but do not type-check the input file
    #[structopt(short = "p", long)]
    parse_only: bool,
    /// Prints code generated from parsed AST (in most cases it's accepted by the parser as well)
    #[structopt(short = "g", long)]
    generated: bool,
    /// Prints errors only
    #[structopt(short = "q", long)]
    quiet: bool,
    /// the input file to type-check (Notice: file should be UTF-8 encoded)
    #[structopt(name = "FILE")]
    file: String,
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

fn read_file(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
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

fn main() {
    let args: CliOptions = CliOptions::from_clap(&app().get_matches());
    if let Some(GenShellSubCommand::Completion { shell }) = args.completion {
        app().gen_completions_to("minittc", shell, &mut io::stdout());
    }

    // If no FILE is specified, return.
    let file_arg = args.file;
    // If cannot read input, return.
    let file_content = match read_file(file_arg.as_str()) {
        Ok(c) => c,
        Err(io_err) => {
            eprintln!("Cannot read `{}`: {}", file_arg, io_err);
            return;
        }
    };
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    let ast = parse_str_err_printed(file_content_utf8).unwrap();
    if !args.quiet {
        println!("Parse successful.");
        if args.generated {
            println!("{}", ast);
        }
    }
    // If parse-only, return before type-checking.
    if args.parse_only {
        return;
    }
    check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
    if !args.quiet {
        println!("Type-check successful.");
    }
}
