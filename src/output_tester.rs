use crate::{
    commands::{set_azure_environment, get_az_cli_command, get_terraform_command},
    resource::{AzureResourceChange, TerraformPlanStep, TerraformResourceChange},
};
use anyhow::Result;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApplicationConfig {
    pub log_level: Option<String>,
    #[serde(rename = "parameters")]
    pub infra_parameters: Vec<InfraParameters>,
    #[serde(rename = "terraformPath")]
    pub terraform_path: Option<String>,
    #[serde(rename = "bicepPath")]
    pub bicep_path: Option<String>,
    pub expected_results: Option<ExpectedResults>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            log_level: Some("info".to_string()),
            infra_parameters: Vec::new(),
            terraform_path: Some("./infra/terraform".to_string()),
            bicep_path: Some("./infra/bicep".to_string()),
            expected_results: None,

        }
    }
}

impl ApplicationConfig {
    pub fn load(path: &Path) -> Result<ApplicationConfig> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        match serde_yaml::from_str(&contents) {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("Error parsing YAML {}", e);
                Ok(ApplicationConfig::default())
            }
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct InfraParameters {
    #[serde(rename = "bicepName")]
    pub bicep_name: Option<String>,
    #[serde(rename = "terraformName")]
    pub terraform_name: Option<String>,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ExpectedResults {
    pub resource_type: String,
    pub resource_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputTester {
    config: ApplicationConfig,
    azure_cli_authenticated: bool,
    location: String,
    bicep_deployment_parameters: Vec<String>,
    terraform_deployment_parameters: Vec<String>,
    bicep_whatif_output: Option<AzureResourceChange>,
    terraform_plan_output: Option<TerraformResourceChange>,
}

impl OutputTester {
    pub fn new() -> Self {
        OutputTester {
            config: ApplicationConfig::default(),
            azure_cli_authenticated: false,
            location: "eastus".to_string(),
            bicep_deployment_parameters: Vec::new(),
            terraform_deployment_parameters: Vec::new(),
            bicep_whatif_output: None,
            terraform_plan_output: None,
        }
    }

    pub fn set_application_config(&mut self, config: ApplicationConfig) -> &mut Self {
        self.config = config;
        self
    }

    pub fn authenticate_azure_cli(&mut self) -> &mut Self {
        match set_azure_environment(None) {
            Ok(_) => self.azure_cli_authenticated = true,
            Err(e) => error!("Error setting Azure environment: {}", e),
        }
        debug!("Azure CLI authenticated: {}", self.azure_cli_authenticated);
        self
    }

    pub fn set_deployment_parameters(&mut self) -> &mut Self {
        debug!("Deployment parameters: {:?}", self.config.infra_parameters);
        for entry in &self.config.infra_parameters {
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
        self
    }

    pub fn execute_bicep_whatif(&mut self) -> &mut Self {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Bicep What If.");
            return self;
        }
        // Execute the bicep whatif command and store the output in self.bicep_whatif_output
        let mut command_arguments = vec![
            "sub",
            "what-if",
            "--location",
            &self.location,
            "--template-file",
            "main.bicep",
            "--no-pretty-print",
            "--output",
            "json",
            "--parameters",
        ];
        // foreach parameter in self.deployment_parameters push onto command_arguments
        for parameter in &self.bicep_deployment_parameters {
            command_arguments.push(parameter);
        }

        let path = PathBuf::from(self.config.bicep_path.as_ref().unwrap());
        let az_bicep = get_az_cli_command("deployment")
            .with_args(command_arguments)
            .with_working_directory(&path)
            .run()
            .expect("Failed to execute Bicep WhatIf command");
        if let Some(output) = az_bicep.get_stdout() {
            self.bicep_whatif_output = serde_json::from_str(&output).unwrap();
            debug!("Bicep WhatIf output: {:?}", self.bicep_whatif_output);
        }
        self
    }

    pub fn init_terraform_environment(&mut self) -> &mut Self {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Terraform Init.");
            return self;
        }
        let path = PathBuf::from(self.config.terraform_path.as_ref().unwrap());
        get_terraform_command("init")
            .with_working_directory(&path)
            .run()
            .expect("Failed to execute Terraform Init command");
        self
    }

    pub fn execute_terraform_plan(&mut self) -> &mut Self {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Terraform Plan.");
            return self;
        }
        
        let path = PathBuf::from(self.config.terraform_path.as_ref().unwrap());

        let mut command_arguments = vec!["-json"];
        for parameter in &self.terraform_deployment_parameters {
            command_arguments.push("-var");
            command_arguments.push(parameter);
        }

        let command = get_terraform_command("plan")
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
}
