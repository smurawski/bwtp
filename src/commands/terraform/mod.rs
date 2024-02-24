use anyhow::Result;
use std::path::PathBuf;
use super::{Command, find_command};


lazy_static! {
    pub static ref TERRAFORM_PATH: PathBuf = get_terraform_path().unwrap();
}

pub fn get_terraform_command(subcommand: &str) -> Command<'static> {
    Command {
        name: "terraform".to_owned(),
        subcommand: subcommand.to_owned(),
        path: TERRAFORM_PATH.clone(),
        args: Vec::new(),
        stdout: None,
        stderr: None,
        exit_status: None,
        verbose: false,
        show_progress: false,
        working_directory: None,
    }
}

fn get_terraform_path() -> Result<PathBuf> {
    let cmd_name = if cfg!(target_os = "windows") {
        "terraform.exe"
    } else {
        "terraform"
    };
    let cli_path = find_command(cmd_name).expect("Failed to find Terraform.  Please install the Terraform to continue.  https://www.terraform.io/downloads.html");
    Ok(cli_path)
}
