#[macro_use]
extern crate lazy_static;

mod output_tester;
mod resource;
mod azcli;

use std::path::Path;

use anyhow::Result;
use output_tester::OutputTester;

lazy_static! {
    pub static ref VERSION: String = format!("v{}", env!("CARGO_PKG_VERSION"));
    //pub static ref VERBOSE: bool = get_app_cli(&VERSION).get_matches().is_present("verbose");
}

fn main() -> Result<()> {
    let test_parameters_file = Path::new("tests/parameters.json");
    OutputTester::new()
        .authenticate_azure_cli()
        .set_deployment_parameters(test_parameters_file)
        .execute_bicep_whatif()
        .execute_terraform_plan()
        .compare_bicep_whatif_and_terraform_plan()
}
