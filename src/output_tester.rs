
use crate::resource::AzureResourceChange;
use anyhow::Result;
use crate::azcli::set_azure_environment;
use std::path::Path;
use log::error;

pub struct OutputTester {
    azure_cli_authenticated: bool,
    deployment_parameters: Vec<String>,
    bicep_whatif_output: Option<AzureResourceChange>,
    terraform_plan_output: Option<AzureResourceChange>,
}

impl OutputTester {
    pub fn new() -> Self {
        OutputTester {
            azure_cli_authenticated: false,
            deployment_parameters: Vec::new(),
            bicep_whatif_output: None,
            terraform_plan_output: None,
        }
    }

    pub fn authenticate_azure_cli(&mut self) -> &mut Self {
        match set_azure_environment(None){
            Ok(_) => self.azure_cli_authenticated = true,
            Err(e) => error!("Error setting Azure environment: {}", e),
        }
        self
    }

    pub fn set_deployment_parameters(&mut self, path: &Path) -> &mut Self {
        self.deployment_parameters = Vec::new();
        self
    }

    pub fn execute_bicep_whatif(&mut self) -> &mut Self {
        // Execute the bicep whatif command and store the output in self.bicep_whatif_output
        self
    }

    pub fn execute_terraform_plan(&mut self) -> &mut Self {
        // Execute the terraform plan command and store the output in self.terraform_plan_output
        self
    }

    pub fn compare_bicep_whatif_and_terraform_plan(&self) -> Result<()> {
        // Compare the bicep whatif and terraform plan outputs
        Ok(())
    }
}