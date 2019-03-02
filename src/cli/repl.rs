use crate::cli::util::ast;
use minitt::type_check::check_main;
use minitt::type_check::TCS;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn repl((mut gamma, mut context): TCS) {
    let mut r = Editor::<()>::new();
    // Load history?
    loop {
        let readline = r.readline(">> ");
        match readline {
            Ok(line) => {
                r.add_history_entry(line.as_ref());
                if line.starts_with(":exit") || line.starts_with(":quit") {
                    break;
                } else if line.starts_with(":gamma") {
                    if gamma.is_empty() {
                        println!("Current Gamma is empty.");
                    } else {
                        println!("Current Gamma:");
                    }
                    gamma
                        .iter()
                        .for_each(|(name, value)| println!("{}: {}", name, value));
                } else if line.starts_with(":telescope") {
                    println!("Current Telescope:");
                } else if line.starts_with(':') {
                    println!("Unrecognized command: {}", line);
                } else {
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
