use std::fmt::{Display, Error, Formatter};
use std::io::{stdin, stdout, Write};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{Context, Editor, Helper};

pub struct MiniHelper {
    pub all_cmd: Vec<String>,
    pub file_completer: FilenameCompleter,
}

impl Completer for MiniHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        if line.starts_with(":load ") || line.starts_with(":l ") {
            return self.file_completer.complete(line, pos, ctx);
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
    fn hint(&self, line: &str, pos: usize, _: &Context<'_>) -> Option<String> {
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

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ReplEnvType {
    Plain,
    Rich,
}

impl Display for ReplEnvType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ReplEnvType::Plain => f.write_str("PLAIN"),
            ReplEnvType::Rich => f.write_str("RICH"),
        }
    }
}

pub fn repl_plain<TCS>(
    mut tcs: TCS,
    prompt: &str,
    welcome_message: impl FnOnce(ReplEnvType) -> (),
    work: impl Fn(TCS, ReplEnvType, &str) -> Option<TCS>,
) {
    welcome_message(ReplEnvType::Plain);
    let stdin = stdin();
    loop {
        print!("{}", prompt);
        stdout().flush().expect("Cannot flush stdout!");
        let mut line = String::new();
        stdin.read_line(&mut line).expect("Cannot flush stdout!");
        if let Some(ok) = work(tcs, ReplEnvType::Plain, line.trim()) {
            tcs = ok;
        } else {
            break;
        };
    }
}

pub fn repl_rich<TCS>(
    mut tcs: TCS,
    prompt: &str,
    create_editor: impl FnOnce() -> Editor<MiniHelper>,
    welcome_message: impl FnOnce(ReplEnvType) -> (),
    work: impl Fn(TCS, ReplEnvType, &str) -> Option<TCS>,
) {
    let mut r = create_editor();
    welcome_message(ReplEnvType::Rich);
    // Load history?
    loop {
        match r.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                r.add_history_entry(line);
                if let Some(ok) = work(tcs, ReplEnvType::Rich, line) {
                    tcs = ok;
                } else {
                    break;
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

pub fn repl<TCS>(
    tcs: TCS,
    prompt: &str,
    repl_kind: ReplEnvType,
    create_editor: impl FnOnce() -> Editor<MiniHelper>,
    welcome_message: impl FnOnce(ReplEnvType) -> (),
    work: impl Fn(TCS, ReplEnvType, &str) -> Option<TCS>,
) {
    use ReplEnvType::*;
    match repl_kind {
        Plain => repl_plain(tcs, prompt, welcome_message, work),
        Rich => repl_rich(tcs, prompt, create_editor, welcome_message, work),
    };
}
