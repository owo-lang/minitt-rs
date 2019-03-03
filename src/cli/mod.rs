/// CLI arguments. Based on structopt (clap)
mod args;

/// File IO. Build AST.
mod util;

/// REPL
mod repl;

pub fn main() {
    use minitt::type_check::{check_main, default_state};
    let args = args::pre();

    // Parse
    let checked = args
        .file
        .clone()
        .and_then(|s| util::ast(s.as_str()))
        .map(|ast| {
            if !args.quiet {
                println!("Parse successful.");
                if args.generated {
                    println!("{}", ast);
                }
            }
            if !args.parse_only {
                // Type Check
                let checked = check_main(ast)
                    .map_err(|err| eprintln!("{}", err))
                    .expect("Type-Check failed.");
                if !args.quiet {
                    println!("Type-Check successful.");
                }
                checked
            } else {
                default_state()
            }
        })
        .unwrap_or_else(|| default_state());

    // REPL
    if args.interactive {
        repl::repl(checked)
    }
}
