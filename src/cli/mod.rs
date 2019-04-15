#[macro_use]
extern crate minitt;

use minitt::parser::{expression_to_expression, parse_str};
use std::str;

/// CLI arguments. Based on structopt (clap)
mod args;

/// File IO. Build AST.
mod util;

/// REPL
mod repl;

pub fn main() {
    use minitt::check::check_main;
    let args = args::pre();

    // Parse
    let checked = args
        .file
        .clone()
        .and_then(|s| util::read_file(s.as_str()))
        .and_then(|s| str::from_utf8(s.as_slice()).ok())
        .and_then(|code| parse_str(code).map_err(|err| eprintln!("{}", err)).ok())
        .map(|tok| {
            if !args.quiet {
                println!("Parse successful.");
            }
            if args.lexical_json {
                println!("{}", tok.to_json())
            }
            let ast = expression_to_expression(tok);
            if args.generated {
                println!("{}", ast);
            }
            if !args.parse_only {
                // Type Check
                let checked = check_main(ast)
                    .map_err(|err| eprintln!("{}", err))
                    .unwrap_or_else(|()| {
                        eprintln!("Type-Check failed.");
                        std::process::exit(1);
                    });
                if !args.quiet {
                    println!("Type-Check successful.");
                }
                checked
            } else {
                Default::default()
            }
        })
        .unwrap_or_else(|| Default::default());

    // REPL
    if args.interactive_plain {
        repl::repl_plain(checked)
    } else if args.interactive {
        repl::repl(checked)
    }
}
