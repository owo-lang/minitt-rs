use crate::cli::util::ast;
use minitt::parser::parse_str_err_printed;
use minitt::syntax::{Expression, GenericTelescope};
use minitt::type_check::{check_contextual, check_infer_contextual, default_state, TCE, TCS};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{CompletionType, Config, Editor, Helper};

struct MiniHelper {
    all_cmd: Vec<String>,
    file_completer: FilenameCompleter,
}

impl Completer for MiniHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        if line.starts_with(LOAD_PFX) {
            return self.file_completer.complete(line, pos);
        }
        Ok((
            0,
            self.all_cmd
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|str| Pair {
                    display: str.clone(),
                    replacement: str.clone(),
                })
                .collect(),
        ))
    }
}

impl Hinter for MiniHelper {
    fn hint(&self, line: &str, pos: usize) -> Option<String> {
        if line.len() < 2 {
            return None;
        }
        self.all_cmd
            .iter()
            .filter(|cmd| cmd.starts_with(line))
            .cloned()
            .map(|cmd| cmd[pos..].to_string())
            .next()
    }
}

impl Highlighter for MiniHelper {}

impl Helper for MiniHelper {}

const PROMPT: &'static str = "=> ";
const QUIT_CMD: &'static str = ":quit";
const GAMMA_CMD: &'static str = ":gamma";
const CTX_CMD: &'static str = ":context";
const HELP_CMD: &'static str = ":help";
const LOAD_CMD: &'static str = ":load";
const TYPE_CMD: &'static str = ":type";

/// Used for REPL command
const LOAD_PFX: &'static str = ":load ";
/// Used for REPL command
const TYPE_PFX: &'static str = ":type ";

pub fn repl(mut tcs: TCS) {
    let all_cmd: Vec<_> = vec![QUIT_CMD, GAMMA_CMD, CTX_CMD, HELP_CMD, LOAD_CMD, TYPE_CMD]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut r = Editor::with_config(
        Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .build(),
    );
    r.set_helper(Some(MiniHelper {
        all_cmd,
        file_completer: FilenameCompleter::new(),
    }));
    // Load history?
    loop {
        match r.readline(PROMPT) {
            Ok(line) => {
                let line = line.trim();
                r.add_history_entry(line.as_ref());
                if line == QUIT_CMD {
                    break;
                } else if line == GAMMA_CMD {
                    show_gamma(&tcs);
                } else if line == CTX_CMD {
                    show_telescope(&tcs);
                } else if line == HELP_CMD {
                    help();
                } else if line.starts_with(LOAD_PFX) {
                    let file = line.trim_start_matches(":load").trim_start();
                    tcs = match ast(file) {
                        Some(ast) => update_tcs(tcs, ast),
                        None => tcs,
                    };
                } else if line.starts_with(TYPE_PFX) {
                    infer(tcs.clone(), line);
                } else if line.starts_with(':') {
                    println!("Unrecognized command: {}", line);
                    println!("Maybe you want to get some `:help`?");
                } else {
                    tcs = match parse_str_err_printed(line).ok() {
                        Some(expr) => update_tcs(tcs, expr),
                        None => tcs,
                    };
                };
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
        };
    }
    // Write history?
}

fn infer(tcs: TCS, line: &str) {
    let file = line.trim_start_matches(":type").trim_start();
    parse_str_err_printed(file)
        .map_err(|()| TCE::Textual("".to_string()))
        .and_then(|ast| check_infer_contextual(tcs, ast))
        .map(|val| println!("{}", val))
        .unwrap_or_else(|err| eprintln!("{}", err));
}

fn help() {
    println!(
        "Interactive minittc {}\n\
         Commands:\n\
         {:<16} {}\n\
         {:<16} {}\n\
         {:<16} {}\n\
         {:<16} {}\n\
         {:<16} {}\n\
         ",
        env!("CARGO_PKG_VERSION"),
        QUIT_CMD,
        "Quit the REPL.",
        GAMMA_CMD,
        "Show current typing context.",
        CTX_CMD,
        "Show current value context.",
        ":load <FILE>",
        "Load an external file.",
        ":type <EXPR>",
        "Try to infer the type of an expression.",
    );
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
        context => {
            println!("Current Telescope:\n{}", context);
        }
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
