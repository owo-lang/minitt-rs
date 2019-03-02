/// CLI arguments. Based on structopt (clap)
pub mod args;

/// File IO and CLI IO.
pub mod io;

use cli::args::*;
use cli::io::*;

use minitt::type_check::check_main;
use rustyline::Editor;
use std::io::stdout;
use structopt::StructOpt;

pub fn main() {
    let args: CliOptions = CliOptions::from_clap(&app().get_matches());
    if let Some(GenShellSubCommand::Completion { shell }) = args.completion {
        app().gen_completions_to("minittc", shell, &mut stdout());
    }

    let file_arg = args.file;
    // Parse
    let ast = match ast(file_arg.as_str()) {
        Some(ast) => ast,
        None => return,
    };
    if !args.quiet {
        println!("Parse successful.");
        if args.generated {
            println!("{}", ast);
        }
    }
    // Type Check
    let _checked = if !args.parse_only {
        check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
        if !args.quiet {
            println!("Type-check successful.");
        }
    } else {
        Default::default()
    };
    // REPL
    if args.interactive {
        let _r = Editor::<()>::new();
    }
}
