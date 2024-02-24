use crate::find_command::find_command;
use anyhow::Result;
use duct::{cmd, ReaderHandle};
use log::{debug, trace};
use std::path::PathBuf;
use std::process::ExitStatus;

lazy_static! {
    pub static ref TERRAFORM_PATH: PathBuf = get_terraform_path().unwrap();
}

pub struct TerraformCommand<'a> {
    name: String,
    path: PathBuf,
    args: Vec<&'a str>,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_status: Option<ExitStatus>,
    verbose: bool,
    show_progress: bool,
    working_directory: Option<PathBuf>,
}

impl<'a> Default for TerraformCommand<'a> {
    fn default() -> TerraformCommand<'a> {
        TerraformCommand {
            name: "plan".to_owned(),
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
}

impl<'a> TerraformCommand<'a> {
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }
    pub fn with_args(mut self, args: Vec<&'a str>) -> Self {
        self.args = args;
        self
    }
    #[allow(dead_code)]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_working_directory(mut self, working_directory: &PathBuf) -> Self {
        self.working_directory = Some(working_directory.clone());
        self
    }

    #[allow(dead_code)]
    pub fn with_show_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }
    pub fn get_stdout(&self) -> Option<String> {
        self.stdout.clone()
    }
    #[allow(dead_code)]
    pub fn get_stderr(&self) -> Option<String> {
        self.stderr.clone()
    }
    pub fn success(&self) -> bool {
        if let Some(s) = &self.exit_status {
            s.success()
        } else {
            false
        }
    }
    pub fn run(mut self) -> Result<Self> {
        trace!("Command: {} running", &self.name);
        debug!("\t`terraform {}`", &self.args.join(" "));
        let mut cmd = cmd(&self.path, &self.args);
        if let Some(working_directory) = &self.working_directory {
            cmd = cmd.dir(working_directory);
        }
        let output = cmd.stderr_capture().stdout_capture().unchecked().run()?;
        self.stdout = Some(String::from_utf8(output.stdout)?);
        self.stderr = Some(String::from_utf8(output.stderr)?);
        self.exit_status = Some(output.status);
        debug!("Terraform command stdout: {:?}", &self.stdout);
        debug!("Terraform command stderr: {:?}", &self.stderr);
        trace!("Finished with command {}", &self.name);

        Ok(self)
    }
    pub fn stderr_reader(&self) -> Result<ReaderHandle> {
        trace!("Command {} running", &self.name);
        debug!("\t`terraform {}`", &self.args.join(" "));
        let reader = cmd(&self.path, &self.args).stderr_capture().reader()?;
        trace!("Returning reader handle.");
        Ok(reader)
    }
    #[allow(dead_code)]
    pub fn stdout_reader(&self) -> Result<ReaderHandle> {
        trace!("Command {} running", &self.name);
        debug!("\t`terraform {}`", &self.args.join(" "));
        let reader = cmd(&self.path, &self.args).stdout_capture().reader()?;
        trace!("Returning reader handle.");
        Ok(reader)
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
