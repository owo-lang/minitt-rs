use crate::cli::util::ast;
use minitt::parser::parse_str_err_printed;
use minitt::syntax::{Expression, GenericTelescope};
use minitt::type_check::{check_contextual, check_infer_contextual, default_state, TCS};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn repl(mut tcs: TCS) {
    let mut r = Editor::<()>::new();
    // Load history?
    loop {
        match r.readline("=> ") {
            Ok(line) => {
                let line = line.trim();
                r.add_history_entry(line.as_ref());
                if line == ":quit" || line == ":q" {
                    break;
                } else if line == ":gamma" || line == ":g" {
                    show_gamma(&tcs);
                } else if line == ":context" || line == ":c" {
                    show_telescope(&tcs)
                } else if line == ":help" || line == ":h" {
                    println!(
                        "Interactive minittc {}\n\
                         Commands:\n\
                         {:<8} {:<9} {}\n\
                         {:<8} {:<9} {}\n\
                         {:<8} {:<9} {}\n\
                         {:<8} {:<9} {}\n\
                         {:<8} {:<9} {}\n\
                         ",
                        env!("CARGO_PKG_VERSION"),
                        ":quit",
                        ":q",
                        "Quit the REPL.",
                        ":gamma",
                        ":g",
                        "Show current typing context.",
                        ":context",
                        ":c",
                        "Show current value context.",
                        ":load",
                        ":l <FILE>",
                        "Load an external file.",
                        ":type",
                        ":t <EXPR>",
                        "Try to infer the type of an expression.",
                    );
                } else if line.starts_with(":load ") || line.starts_with(":l ") {
                    let file = line
                        .trim_start_matches(":l")
                        .trim_start_matches("oad")
                        .trim_start();
                    tcs = match ast(file) {
                        Some(ast) => update_tcs(tcs, ast),
                        None => tcs,
                    }
                } else if line.starts_with(":type ") || line.starts_with(":t ") {
                    let file = line
                        .trim_start_matches(":t")
                        .trim_start_matches("ype")
                        .trim_start();
                    parse_str_err_printed(file)
                        .map_err(|()| "".to_string())
                        .and_then(|ast| check_infer_contextual(tcs.clone(), ast))
                        .map(|val| println!("{}", val))
                        .unwrap_or_else(|err| eprintln!("{}", err));
                } else if line.starts_with(':') {
                    println!("Unrecognized command: {}", line);
                    println!("Maybe you want to get some `:help` or `:h`?");
                } else {
                    tcs = match parse_str_err_printed(line).ok() {
                        Some(expr) => update_tcs(tcs, expr),
                        None => tcs,
                    }
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

fn update_tcs(tcs: TCS, expr: Expression) -> TCS {
    check_contextual(tcs, expr).unwrap_or_else(|err| {
        eprintln!("{}", err);
        eprintln!("Type-Checking State reset due to error (maybe implement recover later).");
        default_state()
    })
}

fn show_telescope(tcs: &TCS) {
    let (_, context) = &tcs;
    match context.as_ref() {
        GenericTelescope::Nil => println!("Current Telescope is empty."),
        context => println!("Current Telescope:"),
    }
}

fn show_gamma(tcs: &TCS) {
    let (gamma, _) = &tcs;
    if gamma.is_empty() {
        println!("Current Gamma is empty.");
    } else {
        println!("Current Gamma:");
    }
    gamma
        .iter()
        .for_each(|(name, value)| println!("{}: {}", name, value));
}
