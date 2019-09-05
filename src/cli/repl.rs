use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use minitt::ast::{Expression, GenericTelescope, Telescope, Value};
use minitt::check::read_back::ReadBack;
use minitt::check::tcm::{TCE, TCS};
use minitt::check::{check_contextual, check_infer_contextual};
use minitt::parser::{parse_str_err_printed, parse_str_to_json};
use minitt_util::repl::{MiniHelper, ReplEnvType};

use crate::util::parse_file;

const PROMPT: &'static str = "=> ";
const QUIT_CMD: &'static str = ":quit";
const GAMMA_CMD: &'static str = ":gamma";
const DEBUG_CMD: &'static str = ":debug";
const CTX_CMD: &'static str = ":context";
const HELP_CMD: &'static str = ":help";
const LOAD_CMD: &'static str = ":load";
const TYPE_CMD: &'static str = ":type";
const INFER_CMD: &'static str = ":infer";
const INFER_DBG_CMD: &'static str = ":infer-debug";
const EVAL_CMD: &'static str = ":eval";
const EVAL_DBG_CMD: &'static str = ":eval-debug";
const LEVEL_CMD: &'static str = ":level";
const LEXICAL_CMD: &'static str = ":lexical";
const NORMALIZE_CMD: &'static str = ":normalize";

/// Used for REPL command
const LOAD_PFX: &'static str = ":load ";
/// Used for REPL command
const TYPE_PFX: &'static str = ":type ";
const INFER_PFX: &'static str = ":infer ";
const INFER_DBG_PFX: &'static str = ":infer-debug ";
const EVAL_PFX: &'static str = ":eval ";
const EVAL_DBG_PFX: &'static str = ":eval-debug ";
const NORMALIZE_PFX: &'static str = ":normalize ";
const LEVEL_PFX: &'static str = ":level ";
const LEXICAL_PFX: &'static str = ":lexical ";

fn repl_work<'a>(tcs: TCS<'a>, current_mode: ReplEnvType, line: &str) -> Option<TCS<'a>> {
    if line == QUIT_CMD {
        None
    } else if line.is_empty() {
        Some(tcs)
    } else if line == GAMMA_CMD {
        show_gamma(&tcs);
        Some(tcs)
    } else if line == CTX_CMD {
        show_telescope(&tcs);
        Some(tcs)
    } else if line == DEBUG_CMD {
        debug(&tcs.context);
        Some(tcs)
    } else if line == HELP_CMD {
        help(current_mode);
        Some(tcs)
    } else if line.starts_with(LOAD_PFX) {
        Some(
            match parse_file(line.trim_start_matches(LOAD_CMD).trim_start(), false) {
                Some(ast) => update_tcs(tcs, ast),
                None => tcs,
            },
        )
    } else if line.starts_with(TYPE_PFX) {
        infer_normalize(
            tcs_borrow!(tcs),
            line.trim_start_matches(TYPE_CMD).trim_start(),
        );
        Some(tcs)
    } else if line.starts_with(INFER_PFX) {
        let line = line.trim_start_matches(INFER_CMD).trim_start();
        infer(tcs_borrow!(tcs), line);
        Some(tcs)
    } else if line.starts_with(INFER_DBG_PFX) {
        let line = line.trim_start_matches(INFER_DBG_CMD).trim_start();
        debug_infer(tcs_borrow!(tcs), line);
        Some(tcs)
    } else if line.starts_with(NORMALIZE_PFX) {
        let line = line.trim_start_matches(NORMALIZE_CMD).trim_start();
        normalize(tcs.context(), line);
        Some(tcs)
    } else if line.starts_with(EVAL_PFX) {
        let line = line.trim_start_matches(EVAL_CMD).trim_start();
        eval(tcs.context(), line);
        Some(tcs)
    } else if line.starts_with(EVAL_DBG_PFX) {
        let line = line.trim_start_matches(EVAL_DBG_CMD).trim_start();
        debug_eval(tcs.context(), line);
        Some(tcs)
    } else if line.starts_with(LEVEL_PFX) {
        let line = line.trim_start_matches(LEVEL_CMD).trim_start();
        level(tcs.context(), line);
        Some(tcs)
    } else if line.starts_with(LEXICAL_PFX) {
        let line = line.trim_start_matches(LEXICAL_CMD).trim_start();
        match parse_str_to_json(line) {
            Err(err) => eprintln!("{}", err),
            Ok(ok) => println!("{}", ok),
        };
        Some(tcs)
    } else if line.starts_with(':') {
        println!("Unrecognized command: {}", line);
        println!("Maybe you want to get some `:help`?");
        Some(tcs)
    } else {
        Some(match parse_str_err_printed(line).ok() {
            Some(expr) => update_tcs(tcs, expr),
            None => tcs,
        })
    }
}

pub fn repl(tcs: TCS) {
    repl_welcome_message(ReplEnvType::Rich);
    let all_cmd: Vec<_> = vec![
        QUIT_CMD,
        GAMMA_CMD,
        DEBUG_CMD,
        CTX_CMD,
        HELP_CMD,
        LOAD_CMD,
        TYPE_CMD,
        INFER_CMD,
        INFER_DBG_CMD,
        NORMALIZE_CMD,
        EVAL_CMD,
        EVAL_DBG_CMD,
        LEVEL_CMD,
        LEXICAL_CMD,
    ]
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
    minitt_util::repl::repl_rich(tcs, PROMPT, &mut r, repl_work);
    // Write history?
}

pub fn repl_plain(tcs: TCS) {
    minitt_util::repl::repl_plain(tcs, PROMPT, repl_welcome_message, repl_work);
}

fn infer_normalize(tcs: TCS, line: &str) {
    infer_impl(tcs, line, |value| println!("{}", value.read_back_please()));
}

fn infer(tcs: TCS, line: &str) {
    infer_impl(tcs, line, |value| println!("{}", value));
}

fn infer_impl(tcs: TCS, line: &str, map: impl FnOnce(Value) -> ()) {
    parse_str_err_printed(line)
        .map_err(|()| TCE::Textual("".to_string()))
        .and_then(|ast| check_infer_contextual(tcs, ast))
        .map(map)
        .unwrap_or_else(|err| eprintln!("{}", err))
}

fn eval(ctx: Telescope, line: &str) {
    eval_impl(ctx, line, |value| println!("{}", value));
}

fn normalize(ctx: Telescope, line: &str) {
    eval_impl(ctx, line, |value| println!("{}", value.read_back_please()));
}

fn level(ctx: Telescope, line: &str) {
    eval_impl(ctx, line, |value: Value| match value.level_safe() {
        Some(level) => println!("{}", level),
        None => println!("The given expression is not a type expression."),
    });
}

fn eval_impl(ctx: Telescope, line: &str, map: impl FnOnce(Value) -> ()) {
    parse_str_err_printed(line)
        .map_err(|()| TCE::Textual("".to_string()))
        .map(|ast| ast.eval(ctx))
        .map(map)
        .unwrap_or_else(|err| eprintln!("{}", err))
}

fn debug(ctx: &Telescope) {
    match &**ctx {
        GenericTelescope::Nil => {}
        GenericTelescope::UpDec(ctx, declaration) => {
            println!("{:?}", declaration);
            debug(ctx);
        }
        GenericTelescope::UpVar(ctx, pattern, value) => {
            println!("var: {} = {:?}", pattern, value);
            debug(ctx);
        }
    }
}

fn debug_eval(ctx: Telescope, line: &str) {
    eval_impl(ctx, line, |value| println!("{:?}", value));
}

fn debug_infer(tcs: TCS, line: &str) {
    infer_impl(tcs, line, |value| println!("{:?}", value));
}

fn repl_welcome_message(current_mode: ReplEnvType) {
    println!(
        "Interactive minittc {}\n\
         Source code: https://github.com/owo-lang/minitt-rs\n\
         Issue tracker: https://github.com/owo-lang/minitt-rs/issues/new\n\n\

         The REPL has two modes: the RICH mode and the PLAIN mode.\n\
         Completion, history command, hints and (in the future) colored output are available in the \
         rich mode, but does not work entirely under Windows PowerShell ISE and Mintty \
         (Cygwin, MinGW and (possibly, depends on your installation) git-bash).\n\
         You are using the {} mode.\n\
         ",
        env!("CARGO_PKG_VERSION"),
        current_mode
    );
}

fn help(current_mode: ReplEnvType) {
    repl_welcome_message(current_mode);
    println!(
        "\
         Commands:\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         {:<20} {}\n\
         ",
        QUIT_CMD,
        "Quit the REPL.",
        GAMMA_CMD,
        "Show current typing context.",
        CTX_CMD,
        "Show current value context.",
        DEBUG_CMD,
        "Show debug-printed value context.",
        ":level <EXPR>",
        "Show the level of the expression, if it's a type.",
        ":lexical <EXPR>",
        "Show the lexical information of the expression.",
        ":load <FILE>",
        "Load an external file.",
        ":infer <EXPR>",
        "Try to infer the type of the given expression.",
        ":infer-debug <EXPR>",
        "Try to infer the type of the given expression and debug-print it.",
        ":type <EXPR>",
        "Try to infer and normalize the type of the given expression.",
        ":eval <EXPR>",
        "Try to evaluate the given expression.",
        ":eval-debug <EXPR>",
        "Try to evaluate the given expression and debug-print it.",
        ":normalize <EXPR>",
        "Try to evaluate and normalize the type of the given expression.",
    );
}

fn update_tcs(tcs: TCS, expr: Expression) -> TCS {
    check_contextual(tcs, expr).unwrap_or_else(|err| {
        eprintln!("{}", err);
        eprintln!("Type-Checking State reset due to error (maybe implement recover later).");
        Default::default()
    })
}

fn show_telescope(tcs: &TCS) {
    match tcs.context.as_ref() {
        GenericTelescope::Nil => println!("Current Telescope is empty."),
        context => {
            println!("Current Telescope:\n{}", context);
        }
    }
}

fn show_gamma(tcs: &TCS) {
    if tcs.gamma.is_empty() {
        println!("Current Gamma is empty.");
    } else {
        println!("Current Gamma:");
    }
    tcs.gamma
        .iter()
        .for_each(|(name, value)| println!("{}: {}", name, value));
}
