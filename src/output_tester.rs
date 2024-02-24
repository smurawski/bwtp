use crate::azcli::set_azure_environment;
use crate::{
    azcli::AzCliCommand,
    resource::{AzureResourceChange, TerraformPlanStep, TerraformResourceChange},
    terraform::TerraformCommand,
};
use anyhow::Result;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputTester {
    azure_cli_authenticated: bool,
    location: String,
    bicep_deployment_parameters: Vec<String>,
    terraform_deployment_parameters: Vec<String>,
    bicep_whatif_output: Option<AzureResourceChange>,
    terraform_plan_output: Option<TerraformResourceChange>,
    parameters: Vec<InfraParameters>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct InfraParameters {
    #[serde(rename = "bicepName")]
    pub bicep_name: Option<String>,
    #[serde(rename = "terraformName")]
    pub terraform_name: Option<String>,
    pub value: String,
}

impl OutputTester {
    pub fn new() -> Self {
        OutputTester {
            azure_cli_authenticated: false,
            location: "eastus".to_string(),
            bicep_deployment_parameters: Vec::new(),
            terraform_deployment_parameters: Vec::new(),
            bicep_whatif_output: None,
            terraform_plan_output: None,
            parameters: Vec::new(),
        }
    }

    pub fn authenticate_azure_cli(&mut self) -> &mut Self {
        match set_azure_environment(None) {
            Ok(_) => self.azure_cli_authenticated = true,
            Err(e) => error!("Error setting Azure environment: {}", e),
        }
        debug!("Azure CLI authenticated: {}", self.azure_cli_authenticated);
        self
    }

    pub fn set_deployment_parameters(&mut self, path: &Path) -> &mut Self {
        // Read the deployment parameters from the file and store them in self.deployment_parameters

        let config = self.read(path).unwrap();
        self.parameters = self.load(&config);
        for entry in &self.parameters {
            if let Some(bicep_name) = &entry.bicep_name {
                if bicep_name == "location" {
                    self.location = entry.value.to_owned();
                }
                self.bicep_deployment_parameters
                    .push(format!("{}={}", bicep_name, entry.value));
            }
            if let Some(terraform_name) = &entry.terraform_name {
                self.terraform_deployment_parameters
                    .push(format!("{}={}", terraform_name, entry.value));
            }
        }

        debug!("Deployment parameters: {:?}", self.parameters);
        self
    }

    pub fn execute_bicep_whatif(&mut self) -> &mut Self {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Bicep What If.");
            return self;
        }
        // Execute the bicep whatif command and store the output in self.bicep_whatif_output
        let mut command_arguments = vec![
            "deployment",
            "sub",
            "what-if",
            "--location",
            &self.location,
            "--template-file",
            "../aks-store-demo/infra/bicep/main.bicep",
            "--no-pretty-print",
            "--output",
            "json",
            "--parameters",
        ];
        // foreach parameter in self.deployment_parameters push onto command_arguments
        for parameter in &self.bicep_deployment_parameters {
            command_arguments.push(parameter);
        }

        let command = AzCliCommand::default()
            .with_name("Bicep WhatIf")
            .with_args(command_arguments)
            .run()
            .expect("Failed to execute Bicep WhatIf command");
        if let Some(output) = command.get_stdout() {
            self.bicep_whatif_output = serde_json::from_str(&output).unwrap();
            debug!("Bicep WhatIf output: {:?}", self.bicep_whatif_output);
        }
        self
    }

    pub fn execute_terraform_plan(&mut self) -> &mut Self {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Terraform Plan.");
            return self;
        }
        // Execute the terraform plan command and store the output in self.terraform_plan_output
        let mut command_arguments = vec!["plan", "-json"];
        for parameter in &self.terraform_deployment_parameters {
            command_arguments.push("-var");
            command_arguments.push(parameter);
        }
        let path = PathBuf::from("../aks-store-demo/infra/terraform");
        TerraformCommand::default()
            .with_name("Terraform Init")
            .with_working_directory(&path)
            .with_args(vec!["init"])
            .run()
            .expect("Failed to execute Terraform Init command");

        let command = TerraformCommand::default()
            .with_name("Terraform Plan")
            .with_working_directory(&path)
            .with_args(command_arguments)
            .run()
            .expect("Failed to execute Terraform Plan command");

        if let Some(output) = command.get_stdout() {
            let result = self.convert_to_terraform_plan(&output);
            self.terraform_plan_output = Some(result);
            debug!("Terraform Plan output: {:?}", self.terraform_plan_output);
        }
        self
    }

    pub fn compare_bicep_whatif_and_terraform_plan(&self) -> Result<()> {
        // Compare the bicep whatif and terraform plan outputs
        Ok(())
    }

    fn convert_to_terraform_plan(&self, output: &str) -> TerraformResourceChange {
        let broken_output: Vec<&str> = output.split("\n").collect();
        let mut result = TerraformResourceChange::default();
        for entry in broken_output {
            debug!("Terraform Plan output: {:?}", entry);
            let temp: TerraformPlanStep = serde_json::from_str(entry).unwrap();
            match temp.record_type.as_str() {
                "version" => {
                    result.version = Some(temp);
                }
                "apply_start" => {
                    result.apply_start.push(temp);
                }
                "apply_complete" => {
                    result.apply_complete.push(temp);
                }
                "planned_change" => {
                    result.planned_change.push(temp);
                }
                "change_summary" => {
                    result.change_summary = Some(temp);
                }
                "outputs" => {
                    result.outputs = Some(temp);
                }
                _ => {
                    error!("Unknown Terraform Plan output: {:?}", temp);
                }
            }
        }
        result
    }

    fn read(&self, path: &Path) -> Result<String, io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    fn load(&self, yaml_str: &str) -> Vec<InfraParameters> {
        match serde_yaml::from_str(&yaml_str) {
            Ok(s) => s,
            Err(e) => {
                error!("Error parsing YAML {}", e);
                Vec::new()
            }
        }
    }
}
