#![allow(dead_code)]
use super::get_az_cli_command;
use anyhow::{Error, Result};
use custom_error::custom_error;
use log::{info, trace};
use regex::Regex;
use serde_json::Value;
use std::io::{BufRead, BufReader};

custom_error! {
    pub AzCliError
    Unknown = "unknown error",
    CliMissing = "Unable to find the Azure CLI.",
    NoParameters = "Command called without required parameters.",
    InvalidJsonError{source: std::string::FromUtf8Error} = "Failed to convert the output.",
    RegexError{source: regex::Error} = "Regex problem.",
    JsonDeserializationError{source: serde_json::Error} = "JSON error",
    CommandFailure{source: std::io::Error} = "Unable to log in via the Azure CLI",
    NotLoggedIn = "Az CLI is not authenticated.",
    MissingTemplate = "No template available to deploy",
    TemplateFailed = "Deployment did not achieve the desired result.",
}

#[derive(Default, Clone, Debug)]
pub struct AzAccountInfo {
    subscription_name: Option<String>,
    subscription_id: Option<String>,
    tenant_id: Option<String>,
}

pub fn set_azure_environment(subscription: Option<&str>) -> Result<()> {
    trace!("Entering set azure environment.");
    info!(
        "Checking to see if the Azure CLI is authenticated and which subscription is default."
    );
    let account = match get_account_info() {
        Ok(a) => a,
        Err(_) => {
            trace!("Failed to get existing login information.  Prompting for new login.");
            login()?;
            info!("Checking for the default subscription.");
            get_account_info()?
        }
    };

    if let Some(account_subscription) = account.subscription_name {
        info!("The default subscription is {}", &account_subscription);

        if let Some(target_subscription) = subscription {
            if account_subscription.trim_matches('"') == target_subscription {
                info!("Subscription already configured correctly.\n");
            } else {
                info!(
                    "Setting the target subscription to {}\n",
                    &target_subscription
                );
                set_target_subscription(target_subscription)?;
            }
        }
    }

    Ok(())
}

fn get_account_info() -> Result<AzAccountInfo> {
    let command = get_az_cli_command("account")
        .with_args(vec!["show", "--output", "json"])
        .run()?;

    let regex_string = "Please run 'az login' to setup account.";
    let re = Regex::new(regex_string)?;

    let account = AzAccountInfo::default();

    let mut return_value = Ok(account);
    let stdout = &command.get_stdout().unwrap();
    if let Some(_captures) = re.captures(stdout) {
        return_value = Err(Error::new(AzCliError::NotLoggedIn));
    } else {
        let v: Value = serde_json::from_str(stdout)?;

        let current_account = AzAccountInfo {
            subscription_id: Some(v["id"].to_string()),
            subscription_name: Some(v["name"].to_string()),
            tenant_id: Some(v["tenantId"].to_string()),
        };

        return_value = Ok(current_account);
    }

    return_value
}

fn login() -> Result<()> {
    let error_pipe_reader = get_az_cli_command("login")
        .stderr_reader()?;

    for line in BufReader::new(error_pipe_reader).lines().flatten() {
        let logged_in_regex = r"^WARNING: (You have logged in\.)";
        let warning_regex = r"^WARNING: (.*)$";
        let warn = Regex::new(warning_regex).expect("Boom");
        let logged_in = Regex::new(logged_in_regex).expect("Boom");

        if let Some(m) = warn.captures(&line) {
            if let Some(m2) = logged_in.captures(&line) {
                info!("{}", &m2[1]);
            } else {
                info!("{}", &m[1]);
            }
        }
    }
    Ok(())
}

fn set_target_subscription(subscription_name: &str) -> Result<()> {
    let command = get_az_cli_command("account")
        .with_args(vec![ "set", "--subscription", subscription_name])
        .run()?;

    if command.success() {
        Ok(())
    } else {
        Err(Error::new(AzCliError::Unknown))
    }
}
