use crate::cli::util::ast;
use minitt::type_check::check_main;
use minitt::type_check::TCS;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use minitt::syntax::GenericTelescope;
use minitt::parser::parse_str_err_printed;
use minitt::type_check::check;
use minitt::type_check::check_contextual;
use std::borrow::Cow;

pub fn repl((mut gamma, mut context): TCS) {
    let mut r = Editor::<()>::new();
    // Load history?
    loop {
        match r.readline(">> ") {
            Ok(line) => {
                r.add_history_entry(line.as_ref());
                if line == ":exit" || line == ":quit" || line == ":q" {
                    break;
                } else if line == ":gamma" {
                    if gamma.is_empty() {
                        println!("Current Gamma is empty.");
                    } else {
                        println!("Current Gamma:");
                    }
                    gamma
                        .iter()
                        .for_each(|(name, value)| println!("{}: {}", name, value));
                } else if line == ":telescope" {
                    match context.as_ref() {
                        GenericTelescope::Nil => {
                            println!("Current Telescope is empty.");
                        }
                        context => {
                            println!("Current Telescope:");
                        }
                    }
                } else if line.starts_with(":load") {
                    // TODO
                } else if line.starts_with(':') {
                    println!("Unrecognized command: {}", line);
                } else if let Some(expr) = parse_str_err_printed(line.as_str()).ok() {
                    match check_contextual((gamma.clone(), context.clone()), expr) {
                        Ok((new_gamma, new_context)) => {
                            gamma = new_gamma;
                            context = new_context;
                        }
                        Err(err) => { eprintln!("{}", err) }
                    }
                    // TODO
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted by Ctrl-c.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Interrupted by Ctrl-d");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    // Write history?
}
