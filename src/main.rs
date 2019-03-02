use clap::{App, Arg, Shell, SubCommand};
use minitt::parser::parse_str_err_printed;
use minitt::type_check::check_main;
use std::io::Read;
use std::str::FromStr;
use std::{fs, io, str};

const BIN_NAME: &'static str = "minittc";

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
    let file_arg = "[FILE] 'the input file to type-check (Notice: file should be UTF-8 encoded)'";
    App::new(BIN_NAME)
        .arg_from_usage("-p --parse-only 'Parse but do not type-check the input file'")
        .arg_from_usage("-g --generated 'Print code generated from parsed AST'")
        .arg_from_usage("-q --quiet 'Do not print anything if no error occurs'")
        .arg_from_usage(file_arg)
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generates completion scripts for your shell to screen")
                .arg(
                    Arg::with_name("SHELL")
                        .required(true)
                        .possible_values(&Shell::variants())
                        .help("The shell you want to use"),
                ),
        )
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .after_help(extra_help)
}

fn main() {
    let matches = app().get_matches();
    match matches.subcommand() {
        ("completions", Some(sub_matches)) => {
            // Should never fail
            let shell = sub_matches.value_of("SHELL").unwrap();
            app().gen_completions_to(
                BIN_NAME,
                // Should never fail
                Shell::from_str(shell).unwrap(),
                &mut io::stdout(),
            );
        }
        _ => (),
    };

    // If no FILE is specified, return.
    let file_arg = match matches.value_of("FILE") {
        None => return,
        Some(a) => a,
    };
    // If cannot read input, return.
    let file_content = match read_file(file_arg) {
        Ok(c) => c,
        Err(io_err) => {
            eprintln!("Cannot read `{}`: {}", file_arg, io_err);
            return;
        }
    };
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    let ast = parse_str_err_printed(file_content_utf8).unwrap();
    let quiet = matches.is_present("quiet");
    if !quiet {
        println!("Parse successful.");
    }
    // If parse-only, return before type-checking.
    if matches.is_present("generated") {
        println!("{}", ast);
    }
    if matches.is_present("parse-only") {
        return;
    }
    check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
    if !quiet {
        println!("Type-check successful.");
    }
}
