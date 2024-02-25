mod azcli;
mod find_command;
mod terraform;

pub use azcli::*;
use duct::{cmd, ReaderHandle};
use anyhow::Result;
use log::{trace, debug};
use std::{path::PathBuf, process::ExitStatus};
pub use find_command::*;
pub use terraform::*;

#[derive(Clone, Debug)]
pub struct Command<'a> {
    name: String,
    subcommand: String,
    path: PathBuf,
    args: Vec<&'a str>,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_status: Option<ExitStatus>,
    verbose: bool,
    show_progress: bool,
    working_directory: Option<PathBuf>,
}

impl<'a> Command<'a> {
    #[allow(dead_code)]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    #[allow(dead_code)]
    pub fn with_subcommand(mut self, subcommand: &str) -> Self {
        self.subcommand = subcommand.to_owned();
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

    pub fn stdout_reader(&self) -> Result<ReaderHandle> {
        trace!("Command: {} {} running", &self.name, &self.subcommand);
        debug!("\t`{} {} {}`", &self.name, &self.subcommand, &self.args.join(" "));
        let mut command_args: Vec<&str> = Vec::new();
        command_args.push(&self.subcommand);
        for arg in &self.args {
            command_args.push(arg);
        }
        let reader = cmd(&self.path, command_args).stderr_capture().reader()?;
        trace!("Returning reader handle.");
        Ok(reader)
    }

    pub fn stderr_reader(&self) -> Result<ReaderHandle> {
        trace!("Command: {} {} running", &self.name, &self.subcommand);
        debug!("\t`{} {} {}`", &self.name, &self.subcommand, &self.args.join(" "));
        let mut command_args: Vec<&str> = Vec::new();
        command_args.push(&self.subcommand);
        for arg in &self.args {
            command_args.push(arg);
        }
        let reader = cmd(&self.path, command_args).stdout_capture().reader()?;
        trace!("Returning reader handle.");
        Ok(reader)
    }

    pub fn run(mut self) -> Result<Self> {
        trace!("Command: {} {} running", &self.name, &self.subcommand);
        debug!("\t`{} {} {}`", &self.name, &self.subcommand, &self.args.join(" "));
        let mut command_args: Vec<&str> = Vec::new();
        command_args.push(&self.subcommand);
        for arg in &self.args {
            command_args.push(*arg);
        }
        let mut command = cmd(&self.path, command_args);
        if let Some(working_directory) = &self.working_directory {
            command = command.dir(working_directory);
        }
        let output = command
            .stderr_capture()
            .stdout_capture()
            .unchecked()
            .run()?;
        self.stdout = Some(String::from_utf8(output.stdout)?);
        self.stderr = Some(String::from_utf8(output.stderr)?);
        self.exit_status = Some(output.status);
        debug!("  Command stdout: {:?}", &self.stdout);
        debug!("  Command stderr: {:?}", &self.stderr);
        trace!("Finished with command {} {}", &self.name, &self.subcommand);

        Ok(self)
    }
}