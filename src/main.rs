use clap::App;
use minitt::parser::parse_str_err_printed;
use minitt::type_check::check_main;
use std::io::Read;
use std::{fs, io, str};

fn read_file(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

fn main() {
    let extra_help = "For extra help please head to \
                      https://github.com/owo-lang/minitt-rs/issues/new";
    let matches = App::new("minittc")
        .arg_from_usage("-p --parse-only 'Parse but do not type-check the input file.'")
        .arg_from_usage("[FILE] 'the input file to type-check'")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .after_help(extra_help)
        .get_matches();
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
    println!("Parse successful.");
    // If parse-only, return before type-checking.
    if matches.is_present("parse-only") {
        return;
    }
    check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
    println!("Type-check successful.");
}
