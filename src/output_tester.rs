use crate::{
    commands::{set_azure_environment, get_az_cli_command, get_terraform_command},
    resource::{AzureResourceChange, TerraformPlanStep, TerraformResourceChange},
};
use anyhow::{anyhow, Result};
use log::{debug, error, warn};
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
    #[serde(rename = "expectedResults")]
    pub expected_results: Vec<ExpectedResults>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            log_level: Some("info".to_string()),
            infra_parameters: Vec::new(),
            terraform_path: Some("./infra/terraform".to_string()),
            bicep_path: Some("./infra/bicep".to_string()),
            expected_results: Vec::new(),

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpectedResults {
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(rename = "name")]
    pub resource_name: Option<String>,
    pub provider: Option<Provider>,
    pub is_expected: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    bicep: bool,
    terraform: bool,
}

impl Provider {
    pub fn new() -> Self {
        Provider {
            bicep: false,
            terraform: false,
        }
    }
    pub fn set_bicep(mut self) -> Self {
        self.bicep = true;
        self
    }
    pub fn set_terraform(mut self) -> Self {
        self.terraform = true;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ActualResults {
    pub expected_results: Vec<ExpectedResults>,
    pub actual_results: Vec<ExpectedResults>,
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
        ];

        if self.bicep_deployment_parameters.is_empty() {
            error!("No Bicep deployment parameters.");
        }
        else {
            command_arguments.push("--parameters");
            for parameter in &self.bicep_deployment_parameters {
                debug!("Bicep WhatIf parameter: {}", parameter);
                command_arguments.push(parameter);
            }
        }

        let path = PathBuf::from(self.config.bicep_path.as_ref().unwrap());
        let az_bicep = get_az_cli_command("deployment")
            .with_args(command_arguments)
            .with_working_directory(&path)
            .run()
            .expect("Failed to execute Bicep WhatIf command");
        if let Some(output) = az_bicep.get_stdout() {
            self.bicep_whatif_output = serde_json::from_str(&output).unwrap();
            debug!("Bicep WhatIf output: {:#?}", self.bicep_whatif_output);
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
            debug!("Terraform Plan parameter: {}", parameter);
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
            debug!("Terraform Plan output: {:#?}", self.terraform_plan_output);
        }
        
        self
    }

    pub fn compare_bicep_whatif_and_terraform_plan(&self) -> Result<()> {
        if !self.azure_cli_authenticated {
            error!("Azure CLI not authenticated. Skipping Comparison.");
            return Err(anyhow!("Azure CLI not authenticated. Skipping Comparison."));
        }
        // If the expected results are not set, then we can't compare
        if self.config.expected_results.is_empty() {
            warn!("Expected results not set. Skipping comparison.");
            return Ok(());
        }

        // Compare the bicep whatif and terraform plan outputs
        let mut azure_resources = self.get_bicep_resources_for_comparison();
        let mut terraform_resources = self.get_terraform_resources_for_comparison();

        let mut response = ActualResults::default();
        for expected in &self.config.expected_results {
            let actual_result = self.process_expected_results(expected, &mut azure_resources, &mut terraform_resources);
            response.actual_results.push(actual_result);
        }
        
        terraform_resources.iter().for_each(|x| {
            let unexpected_result = self.process_unexpected_terraform_results(&mut azure_resources, x);
            response.actual_results.push(unexpected_result);
        });

        azure_resources.iter().for_each(|x| {
            let unexpected_result = self.process_unexpected_bicep_results(&mut terraform_resources, x);
            response.actual_results.push(unexpected_result);
        });

        response.expected_results = self.config.expected_results.clone();
        println!("Results: {} ", serde_json::to_string_pretty(&response).unwrap() );

        Ok(())
    }

    fn process_unexpected_bicep_results(&self,  terraform_resources: &mut Vec<String>, x: &String) -> ExpectedResults {
        let mut unexpected_provider = Provider::new().set_bicep();
        if terraform_resources.contains(x) {
            unexpected_provider = unexpected_provider.set_terraform();
            self.remove_matched_resource(x,  terraform_resources)
        }
        let unexpected_result = ExpectedResults {
            resource_type: x.to_string(),
            resource_name: None,
            provider: Some(unexpected_provider),
            is_expected: Some(false),
        };
        unexpected_result
    }

    fn process_unexpected_terraform_results(&self, azure_resources: &mut Vec<String>, x: &String) -> ExpectedResults {
        let mut unexpected_provider = Provider::new().set_terraform();
        if azure_resources.contains(x) {
            unexpected_provider = unexpected_provider.set_bicep();
            self.remove_matched_resource(x, azure_resources)
        }
        let unexpected_result = ExpectedResults {
            resource_type: x.to_string(),
            resource_name: None,
            provider: Some(unexpected_provider),
            is_expected: Some(false),
        };
        unexpected_result
    }

    fn process_expected_results(&self, expected: &ExpectedResults, azure_resources: &mut Vec<String>, terraform_resources: &mut Vec<String>) -> ExpectedResults {
        let mut actual_result = expected.clone();
        actual_result.is_expected = Some(true);
        let mut provider = Provider::new();
        if azure_resources.contains(&expected.resource_type) {
            provider = provider.set_bicep();
            self.remove_matched_resource(&expected.resource_type, azure_resources);
        }
        if terraform_resources.contains(&expected.resource_type) {
            provider = provider.set_terraform();
            self.remove_matched_resource(&expected.resource_type, terraform_resources);
        }
        actual_result.provider = Some(provider);
        actual_result
    }

    fn get_terraform_resources_for_comparison(&self) -> Vec<String> {
        self.terraform_plan_output
            .as_ref()
            .unwrap()
            .planned_change
            .iter()
            .map(|x| x.change.as_ref().unwrap().resource.get_comparison_resource().to_string())
            .collect::<Vec<String>>()
    }

    fn get_bicep_resources_for_comparison(&self) -> Vec<String> {
        self.bicep_whatif_output
            .as_ref()
            .unwrap()
            .changes
            .iter()
            .map(|x| x.after.as_ref().unwrap().get_comparison_resource()).collect::<Vec<String>>()
    }

    fn remove_matched_resource(&self, resource: &str, vec: &mut Vec<String>) {
        if let Some(index) = vec.iter().position(|value| *value == resource) {
            vec.swap_remove(index);
        }
    }

    fn convert_to_terraform_plan(&self, output: &str) -> TerraformResourceChange {
        let broken_output: Vec<&str> = output.split("\n").collect();
        let mut result = TerraformResourceChange::default();
        for entry in broken_output {
            debug!("Terraform Plan output: {:?}", entry);
            if str::is_empty(entry) {
                continue;
            }
            let temp: TerraformPlanStep = serde_json::from_str(entry).unwrap();
            match temp.record_type.as_str() {
                "version" => {
                    debug!("Setting Terraform Plan Version");
                    result.version = Some(temp);
                }
                "apply_start" => {
                    debug!("Setting Terraform Plan Apply Start");
                    result.apply_start.push(temp);
                }
                "apply_complete" => {
                    debug!("Setting Terraform Plan Apply Complete");
                    result.apply_complete.push(temp);
                }
                "planned_change" => {
                    debug!("Setting Terraform Plan Planned Change");
                    result.planned_change.push(temp);
                }
                "change_summary" => {
                    debug!("Setting Terraform Plan Change Summary");
                    result.change_summary = Some(temp);
                }
                "outputs" => {
                    debug!("Setting Terraform Plan Outputs");
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

#[cfg(test)]
mod test {
    use super::*;
    // use crate::resource::{AzureResourceChange, TerraformResourceChange};

    #[test]
    pub fn test_load_application_config() {
        let path = Path::new("tests/parameters.yaml");
        let result = ApplicationConfig::load(path).unwrap();
        assert_eq!(result.infra_parameters.len(), 4);
        assert_eq!(result.terraform_path, Some("../aks-store-demo/infra/terraform".to_string()));
        assert_eq!(result.bicep_path, Some("../aks-store-demo/infra/bicep".to_string()));
        assert_eq!(result.expected_results.len(), 3);
    }

    #[test]
    pub fn test_set_deployment_parameters() {
        let mut tester = OutputTester::new();
        let path = Path::new("tests/parameters.yaml");
        let config = ApplicationConfig::load(path).unwrap();
        tester.set_application_config(config);
        tester.set_deployment_parameters();
        assert_eq!(tester.location, "eastus");
        assert_eq!(tester.bicep_deployment_parameters.len(), 3);
        assert_eq!(tester.terraform_deployment_parameters.len(), 2);
    }

    // #[test]
    // pub fn test_compare_bicep_whatif_and_terraform_plan() {
    //     let mut tester = OutputTester::new();
    //     let path = Path::new("tests/parameters.yaml");
    //     let config = ApplicationConfig::load(path).unwrap();
    //     tester.set_application_config(config);

    //     let tester.bicep

    //     tester.compare_bicep_whatif_and_terraform_plan().unwrap();
    // }

}