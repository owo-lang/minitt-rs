use std::str;

use minitt::ast::Expression;
use minitt::parser::{expression_to_expression, parse_str, Tok};
use minitt_util::io::read_file;

pub fn parse_file(file_arg: &str, print_lexical_json: bool) -> Option<Expression> {
    // If cannot read input, return.
    let file_content = read_file(file_arg)?;
    // Read file
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    // Parse
    let tok: Tok = parse_str(file_content_utf8)
        .map_err(|err| eprintln!("{}", err))
        .ok()?;
    if print_lexical_json {
        println!("{}", tok.to_json());
    }
    Some(expression_to_expression(tok))
}
