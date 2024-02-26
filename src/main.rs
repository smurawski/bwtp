#[macro_use]
extern crate lazy_static;

mod commands;
mod output_tester;
mod resource;

use std::{env::var, path::Path};

use anyhow::Result;
use env_logger::Env;
use output_tester::{ApplicationConfig, OutputTester};

lazy_static! {
    pub static ref VERSION: String = format!("v{}", env!("CARGO_PKG_VERSION"));
    //pub static ref VERBOSE: bool = get_app_cli(&VERSION).get_matches().is_present("verbose");
}

fn main() -> Result<()> {
    let config_file_path = Path::new("tests/parameters.yaml");
    let application_config = ApplicationConfig::load(config_file_path)?;

    let log_level = application_config.log_level.as_deref().unwrap_or("warn");
    env_logger::init_from_env(
        Env::default().default_filter_or(log_level)
    );

    if let Ok(only_config) = var("CONFIG") {
        let print_config = match only_config.to_lowercase().as_str() {
            "true" | "1" => true,
            _ => false,
        };
        
        if print_config {
            println!("{:#?}", &application_config);
            return Ok(());
        }
    }

    OutputTester::new()
        .set_application_config(application_config)
        .authenticate_azure_cli()
        .set_deployment_parameters()
        .execute_bicep_whatif()
        .init_terraform_environment()
        .execute_terraform_plan()
        .compare_bicep_whatif_and_terraform_plan()
}
