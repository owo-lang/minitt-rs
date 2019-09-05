#[macro_use]
extern crate minitt;

use minitt_util::repl::ReplEnvType;

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
        .and_then(|s| util::parse_file(s.as_str(), args.lexical_json))
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
    repl::repl(
        checked,
        if args.interactive_plain {
            Some(ReplEnvType::Plain)
        } else if args.interactive {
            Some(ReplEnvType::Rich)
        } else {
            None
        },
    );
}
