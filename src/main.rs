#[macro_use]
extern crate lazy_static;

mod azcli;
mod find_command;
mod output_tester;
mod resource;
mod terraform;

use std::path::Path;

use anyhow::Result;
use env_logger::Env;
use output_tester::OutputTester;

lazy_static! {
    pub static ref VERSION: String = format!("v{}", env!("CARGO_PKG_VERSION"));
    //pub static ref VERBOSE: bool = get_app_cli(&VERSION).get_matches().is_present("verbose");
}

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let test_parameters_file = Path::new("tests/parameters.yaml");
    OutputTester::new()
        .authenticate_azure_cli()
        .set_deployment_parameters(test_parameters_file)
        .execute_bicep_whatif()
        .execute_terraform_plan()
        .compare_bicep_whatif_and_terraform_plan()
}
