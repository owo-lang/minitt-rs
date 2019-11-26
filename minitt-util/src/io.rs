use std::fs::create_dir_all;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io, str};

fn read_file_impl(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

pub fn read_file(file_arg: &str) -> Option<Vec<u8>> {
    read_file_impl(file_arg)
        .map_err(|io_err| eprintln!("Cannot read `{}`: {}", file_arg, io_err))
        .ok()
}

const FAIL_CREATE_DEFAULT: &str = "Failed to create default working file";

pub fn default_config_dir() -> io::Result<PathBuf> {
    let agda_tac_dir = dirs::home_dir().expect(FAIL_CREATE_DEFAULT).join(".dtlc");
    create_dir_all(&agda_tac_dir)?;
    Ok(agda_tac_dir)
}

pub fn history_file(lang_name: &str) -> io::Result<PathBuf> {
    Ok(default_config_dir()?.join(&format!(".{}_repl_history", lang_name)))
}
