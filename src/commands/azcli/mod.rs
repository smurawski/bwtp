#![allow(unused_assignments)]

mod cli;

use super::{Command, find_command};
use anyhow::Result;

use std::path::PathBuf;


pub use cli::*;



lazy_static! {
    pub static ref AZ_CLI_PATH: PathBuf = get_az_cli_path().unwrap();
}

pub fn get_az_cli_command(subcommand: &str) -> Command<'static> {
    Command {
        name: "az".to_owned(),
        subcommand: subcommand.to_owned(),
        path: AZ_CLI_PATH.clone(),
        args: Vec::new(),
        stdout: None,
        stderr: None,
        exit_status: None,
        verbose: false,
        show_progress: false,
        working_directory: None,
    }
}

fn get_az_cli_path() -> Result<PathBuf> {
    let cmd_name = if cfg!(target_os = "windows") {
        "az.cmd"
    } else {
        "az"
    };
    let cli_path = find_command(cmd_name).expect("Failed to find the Az CLI.  Please install the Az CLI to continue (https://aka.ms/containerapps/install-az-cli) or use --skip-azure to only process the Compose files.");
    Ok(cli_path)
}
