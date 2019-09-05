use clap::{App, Shell};
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum GenShellSubCommand {
    /// Prints completion scripts for your shell
    Completion {
        /// Prints completion scripts for your shell
        #[structopt(
            name = "generate-completion-script-for",
            alias = "gcf",
            possible_values(&Shell::variants()),
            case_insensitive(true)
        )]
        shell: Shell,
    },
}

pub fn cli_completion_generation(
    completion: &Option<GenShellSubCommand>,
    app: impl Fn() -> App<'static, 'static>,
) {
    if let Some(GenShellSubCommand::Completion { shell }) = completion {
        let mut app = app();
        let name = app.get_name().to_owned();
        app.gen_completions_to(&name, *shell, &mut std::io::stdout());
    }
}
