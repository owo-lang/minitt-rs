use clap::{App, Arg};
use minitt::parser::parse_str_err_printed;
use minitt::type_check::check_main;
use std::io::Read;
use std::{fs, io, str};

fn check_file(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

fn main() {
    let matches = App::new("minittc")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::from_usage("[FILE] 'the input file to type-check'"))
        .get_matches();
    // If no FILE is specified, return.
    let file_arg = matches.value_of("FILE").unwrap();
    let file_content = check_file(file_arg).unwrap();
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    let ast = parse_str_err_printed(file_content_utf8).unwrap();
    println!("Parse successful.");
    check_main(ast).map_err(|err| eprintln!("{}", err)).unwrap();
    println!("Type-check successful.");
}
