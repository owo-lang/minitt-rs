mod cli;
use cli::args::*;

use minitt::parser::parse_str_err_printed;
use minitt::type_check::check_main;
use rustyline::Editor;
use std::io::Read;
use std::{fs, io, str};
use structopt::StructOpt;

fn read_file(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
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
    // Read file
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    // Parse
    let ast = parse_str_err_printed(file_content_utf8).unwrap();
    if !args.quiet {
        println!("Parse successful.");
        if args.generated {
            println!("{}", ast);
        }
    }
    // Type Check
    let checked = if !args.parse_only {
        check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
        if !args.quiet {
            println!("Type-check successful.");
        }
    } else {
        Default::default()
    };
    // REPL
    if args.interactive {
        let r = Editor::<()>::new();
    }
}
