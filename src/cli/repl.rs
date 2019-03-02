use minitt::parser::parse_str_err_printed;
use minitt::syntax::GenericTelescope;
use minitt::type_check::{check_contextual, default_state, TCS};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn repl(mut tcs: TCS) {
    let mut r = Editor::<()>::new();
    // Load history?
    loop {
        match r.readline(">> ") {
            Ok(line) => {
                let line = line.trim();
                r.add_history_entry(line.as_ref());
                if line == ":exit" || line == ":quit" || line == ":q" {
                    break;
                } else if line == ":gamma" || line == ":g" {
                    let (gamma, _) = &tcs;
                    if gamma.is_empty() {
                        println!("Current Gamma is empty.");
                    } else {
                        println!("Current Gamma:");
                    }
                    gamma
                        .iter()
                        .for_each(|(name, value)| println!("{}: {}", name, value));
                } else if line == ":context" || line == ":c" {
                    let (_, context) = &tcs;
                    match context.as_ref() {
                        GenericTelescope::Nil => println!("Current Telescope is empty."),
                        context => println!("Current Telescope:"),
                    }
                } else if line.starts_with(":load ") || line.starts_with(":l ") {
                    // TODO
                } else if line.starts_with(':') {
                    println!("Unrecognized command: {}", line);
                } else if let Some(expr) = parse_str_err_printed(line).ok() {
                    match check_contextual(tcs, expr) {
                        Ok(new_tcs) => tcs = new_tcs,
                        Err(err) => {
                            tcs = default_state();
                            eprintln!("{}", err);
                            eprintln!("Type-Checking State reset due to error (maybe implement recover later).")
                        }
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
