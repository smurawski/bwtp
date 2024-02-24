use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct AzureResourceChange {
    changes: Vec<AzureResourceChangeDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum AzureResourceChangeType {
    Create,
    Delete,
    Update,
    Unsupported,
}

impl Default for AzureResourceChangeType {
    fn default() -> Self {
        AzureResourceChangeType::Create
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct AzureResourceChangeDetail {
    after: Option<AzureResource>,
    before: Option<AzureResource>,
    #[serde(rename = "changeType")]
    change_type: AzureResourceChangeType,
    delta: Option<String>,
    #[serde(rename = "resourceId")]
    resource_id: String,
    #[serde(rename = "unsupportedReason")]
    unsupported_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct AzureResource {
    #[serde(rename = "apiVersion")]
    api_version: String,
    id: String,
    location: Option<String>,
    name: String,
    tags: Option<HashMap<String, String>>,
    #[serde(rename = "type")]
    resource_type: String,
}

#[cfg(test)]
mod azure_test {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_azure_resource_change() {
        let json = r#"
        {
            "changes": [
                {
                    "after": {
                        "apiVersion": "2021-04-01",
                        "id": "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore",
                        "location": "eastus",
                        "name": "rg-nevermore",
                        "tags": {
                            "azd-env-name": "nevermore"
                        },
                        "type": "Microsoft.Resources/resourceGroups"
                    },
                    "before": null,
                    "changeType": "Create",
                    "delta": null,
                    "resourceId": "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore",
                    "unsupportedReason": null
                },
                {
                    "after": {
                        "apiVersion": "2021-04-01",
                        "id": "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore/providers/Microsoft.ContainerService/managedClusters/aks-nevermore",
                        "location": "eastus",
                        "name": "aks-nevermore",
                        "tags": {
                            "azd-env-name": "nevermore"
                        },
                        "type": "Microsoft.ContainerService/managedClusters"
                    },
                    "before": null,
                    "changeType": "Create",
                    "delta": null,
                    "resourceId": "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore/providers/Microsoft.ContainerService/managedClusters/aks-nevermore",
                    "unsupportedReason": null
                }
            ]
        }
        "#;
        let changes: AzureResourceChange = serde_json::from_str(json).unwrap();
        assert_eq!(*&changes.changes.len(), 2);
        assert_eq!(
            &changes.changes[0].resource_id,
            "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore"
        );
        assert_eq!(
            *&changes.changes[0].change_type,
            AzureResourceChangeType::Create
        );
        assert_eq!(
            &changes.changes[0].after.as_ref().unwrap().api_version,
            "2021-04-01"
        );
        assert_eq!(
            &changes.changes[0].after.as_ref().unwrap().id,
            "/subscriptions/13ae0661-466f-4189-9095-cbd2e68a485f/resourceGroups/rg-nevermore"
        );
        assert_eq!(
            *&changes.changes[0]
                .after
                .as_ref()
                .unwrap()
                .location
                .as_ref()
                .unwrap(),
            "eastus"
        );
        assert_eq!(
            &changes.changes[0].after.as_ref().unwrap().name,
            "rg-nevermore"
        );
        assert_eq!(
            *&changes.changes[0]
                .after
                .as_ref()
                .unwrap()
                .tags
                .as_ref()
                .unwrap()
                .get("azd-env-name")
                .unwrap(),
            "nevermore"
        );
        assert_eq!(
            &changes.changes[0].after.as_ref().unwrap().resource_type,
            "Microsoft.Resources/resourceGroups"
        );
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformResourceChange {
    pub version: Option<TerraformPlanStep>,
    pub apply_start: Vec<TerraformPlanStep>,
    pub apply_complete: Vec<TerraformPlanStep>,
    pub planned_change: Vec<TerraformPlanStep>,
    pub change_summary: Option<TerraformPlanStep>,
    pub outputs: Option<TerraformPlanStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformPlanStep {
    #[serde(rename = "type")]
    pub record_type: String,
    #[serde(rename = "@level")]
    pub level: String,
    #[serde(rename = "@message")]
    pub message: String,
    #[serde(rename = "@module")]
    pub module: String,
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub terraform: Option<String>,
    pub ui: Option<String>,
    pub hook: Option<TerraformHook>,
    pub id_key: Option<String>,
    pub id_value: Option<String>,
    pub elapsed_seconds: Option<u32>,
    pub change: Option<TerraformHook>,
    pub changes: Option<TerraformChanges>,
    pub outputs: Option<HashMap<String, TerraformOutput>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformHook {
    resource: TerraformResource,
    action: String,
    id_key: Option<String>,
    id_value: Option<String>,
    elapsed_seconds: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformResource {
    addr: String,
    module: String,
    resource: String,
    implied_provider: String,
    resource_type: String,
    resource_name: String,
    resource_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformChanges {
    add: u32,
    change: u32,
    import: u32,
    remove: u32,
    operation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TerraformOutput {
    sensitive: bool,
    action: String,
}

//tests module
#[cfg(test)]
mod terraform_test {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_terraform_version() {
        let json = r#"
        {
            "@level": "info",
            "@message": "Terraform 1.6.5",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:49:28.479064-06:00",
            "terraform": "1.6.5",
            "type": "version",
            "ui": "1.2"
        }
        "#;
        let version: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(version.record_type, "version");
        assert_eq!(version.level, "info");
        assert_eq!(version.message, "Terraform 1.6.5");
        assert_eq!(version.module, "terraform.ui");
        assert_eq!(version.timestamp, "2024-02-23T13:49:28.479064-06:00");
        assert_eq!(version.terraform.unwrap(), "1.6.5");
        assert_eq!(version.ui.unwrap(), "1.2");
    }

    #[test]
    fn test_deserialize_terraform_apply_start() {
        let json = r#"
        {
            "@level": "info",
            "@message": "data.azurerm_client_config.current: Refreshing...",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:50:03.507371-06:00",
            "hook": {
                "resource": {
                    "addr": "data.azurerm_client_config.current",
                    "module": "",
                    "resource": "data.azurerm_client_config.current",
                    "implied_provider": "azurerm",
                    "resource_type": "azurerm_client_config",
                    "resource_name": "current",
                    "resource_key": null
                },
                "action": "read"
            },
            "type": "apply_start"
        }
        "#;
        let apply_start: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(apply_start.record_type, "apply_start");
        assert_eq!(apply_start.level, "info");
        assert_eq!(
            apply_start.message,
            "data.azurerm_client_config.current: Refreshing..."
        );
        assert_eq!(apply_start.module, "terraform.ui");
        assert_eq!(apply_start.timestamp, "2024-02-23T13:50:03.507371-06:00");

        let hook = apply_start.hook.unwrap();
        assert_eq!(&hook.resource.addr, "data.azurerm_client_config.current");
        assert_eq!(&hook.resource.module, "");
        assert_eq!(
            &hook.resource.resource,
            "data.azurerm_client_config.current"
        );
        assert_eq!(&hook.resource.implied_provider, "azurerm");
        assert_eq!(&hook.resource.resource_type, "azurerm_client_config");
        assert_eq!(&hook.resource.resource_name, "current");
        assert_eq!(&hook.action, "read");
        assert!(&hook.resource.resource_key.is_none());
    }

    #[test]
    fn test_deserialize_terraform_apply_complete() {
        let json = r#"
        {
            "@level": "info",
            "@message": "data.azurerm_client_config.current: Refresh complete after 0s [id=Y2xpZW50Q29uZmlncy9jbGllbnRJZD0wNGIwNzc5NS04ZGRiLTQ2MWEtYmJlZS0wMmY5ZTFiZjdiNDY7b2JqZWN0SWQ9YmIwOTk4MDctMGI5ZC00YzYzLTk1YWMtZDg2ZjM4MjQ4ZjYyO3N1YnNjcmlwdGlvbklkPTEzYWUwNjYxLTQ2NmYtNDE4OS05MDk1LWNiZDJlNjhhNDg1Zjt0ZW5hbnRJZD03MmY5ODhiZi04NmYxLTQxYWYtOTFhYi0yZDdjZDAxMWRiNDc=]",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:50:03.508973-06:00",
            "hook": {
                "resource": {
                    "addr": "data.azurerm_client_config.current",
                    "module": "",
                    "resource": "data.azurerm_client_config.current",
                    "implied_provider": "azurerm",
                    "resource_type": "azurerm_client_config",
                    "resource_name": "current",
                    "resource_key": null
                },
                "action": "read",
                "id_key": "id",
                "id_value": "Y2xpZW50Q29uZmlncy9jbGllbnRJZD0wNGIwNzc5NS04ZGRiLTQ2MWEtYmJlZS0wMmY5ZTFiZjdiNDY7b2JqZWN0SWQ9YmIwOTk4MDctMGI5ZC00YzYzLTk1YWMtZDg2ZjM",
                "elapsed_seconds": 0
            },
            "type": "apply_complete"
        }
        "#;
        let apply_complete: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(apply_complete.record_type, "apply_complete");
        assert_eq!(apply_complete.level, "info");
        assert_eq!(apply_complete.message, "data.azurerm_client_config.current: Refresh complete after 0s [id=Y2xpZW50Q29uZmlncy9jbGllbnRJZD0wNGIwNzc5NS04ZGRiLTQ2MWEtYmJlZS0wMmY5ZTFiZjdiNDY7b2JqZWN0SWQ9YmIwOTk4MDctMGI5ZC00YzYzLTk1YWMtZDg2ZjM4MjQ4ZjYyO3N1YnNjcmlwdGlvbklkPTEzYWUwNjYxLTQ2NmYtNDE4OS05MDk1LWNiZDJlNjhhNDg1Zjt0ZW5hbnRJZD03MmY5ODhiZi04NmYxLTQxYWYtOTFhYi0yZDdjZDAxMWRiNDc=]");
        assert_eq!(apply_complete.module, "terraform.ui");
        assert_eq!(apply_complete.timestamp, "2024-02-23T13:50:03.508973-06:00");
        let hook = apply_complete.hook.unwrap();
        assert_eq!(&hook.resource.addr, "data.azurerm_client_config.current");
        assert_eq!(&hook.resource.module, "");
        assert_eq!(
            &hook.resource.resource,
            "data.azurerm_client_config.current"
        );
        assert_eq!(&hook.resource.implied_provider, "azurerm");
        assert_eq!(&hook.resource.resource_type, "azurerm_client_config");
        assert_eq!(&hook.resource.resource_name, "current");
        assert!(&hook.resource.resource_key.is_none());
        assert_eq!(&hook.action, "read");
        assert_eq!(&hook.id_key.unwrap(), "id");
        assert_eq!(&hook.id_value.unwrap(), "Y2xpZW50Q29uZmlncy9jbGllbnRJZD0wNGIwNzc5NS04ZGRiLTQ2MWEtYmJlZS0wMmY5ZTFiZjdiNDY7b2JqZWN0SWQ9YmIwOTk4MDctMGI5ZC00YzYzLTk1YWMtZDg2ZjM");
        assert_eq!(*&hook.elapsed_seconds.unwrap(), 0);
    }

    #[test]
    fn test_deserialize_terraform_planned_change() {
        let json = r#"
        {
            "@level": "info",
            "@message": "random_integer.example: Plan to create",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:50:04.650549-06:00",
            "change": {
                "resource": {
                    "addr": "random_integer.example",
                    "module": "",
                    "resource": "random_integer.example",
                    "implied_provider": "random",
                    "resource_type": "random_integer",
                    "resource_name": "example",
                    "resource_key": null
                },
                "action": "create"
            },
            "type": "planned_change"
        }
        "#;
        let planned_change: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(planned_change.record_type, "planned_change");
        assert_eq!(planned_change.level, "info");
        assert_eq!(
            planned_change.message,
            "random_integer.example: Plan to create"
        );
        assert_eq!(planned_change.module, "terraform.ui");
        assert_eq!(planned_change.timestamp, "2024-02-23T13:50:04.650549-06:00");
        let changes = planned_change.change.unwrap();
        assert_eq!(&changes.resource.addr, "random_integer.example");
        assert_eq!(&changes.resource.module, "");
        assert_eq!(&changes.resource.resource, "random_integer.example");
        assert_eq!(&changes.resource.implied_provider, "random");
        assert_eq!(&changes.resource.resource_type, "random_integer");
        assert_eq!(&changes.resource.resource_name, "example");
        assert!(&changes.resource.resource_key.is_none());
        assert_eq!(&changes.action, "create");
    }

    #[test]
    fn test_deserialize_terraform_change_summary() {
        let json = r#"
        {
            "@level": "info",
            "@message": "Plan: 5 to add, 0 to change, 0 to destroy.",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:50:04.652705-06:00",
            "changes": {
                "add": 5,
                "change": 0,
                "import": 0,
                "remove": 0,
                "operation": "plan"
            },
            "type": "change_summary"
        }
        "#;
        let change_summary: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(change_summary.record_type, "change_summary");
        assert_eq!(change_summary.level, "info");
        assert_eq!(
            change_summary.message,
            "Plan: 5 to add, 0 to change, 0 to destroy."
        );
        assert_eq!(change_summary.module, "terraform.ui");
        assert_eq!(change_summary.timestamp, "2024-02-23T13:50:04.652705-06:00");
        let changes = change_summary.changes.unwrap();
        assert_eq!(*&changes.add, 5);
        assert_eq!(*&changes.change, 0);
        assert_eq!(*&changes.import, 0);
        assert_eq!(*&changes.remove, 0);
        assert_eq!(&changes.operation, "plan");
    }

    #[test]
    fn test_deserialize_terraform_outputs() {
        let json = r#"
        {
            "@level": "info",
            "@message": "Outputs: 25",
            "@module": "terraform.ui",
            "@timestamp": "2024-02-23T13:50:04.652705-06:00",
            "outputs": {
                "AZURE_AKS_CLUSTER_ID": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_AKS_CLUSTER_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_AKS_CLUSTER_NODE_RESOURCEGROUP_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_AKS_NAMESPACE": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_AKS_OIDC_ISSUER_URL": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_COSMOS_DATABASE_KEY": {
                    "sensitive": true,
                    "action": "create"
                },
                "AZURE_COSMOS_DATABASE_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_COSMOS_DATABASE_URI": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_DATABASE_API": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_IDENTITY_CLIENT_ID": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_KEY_VAULT_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_OPENAI_ENDPOINT": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_OPENAI_KEY": {
                    "sensitive": true,
                    "action": "create"
                },
                "AZURE_OPENAI_MODEL_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_REGISTRY_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_REGISTRY_URI": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_RESOURCE_GROUP_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_RESOURCENAME_SUFFIX": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_HOST": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_LISTENER_KEY": {
                    "sensitive": true,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_LISTENER_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_SENDER_KEY": {
                    "sensitive": true,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_SENDER_NAME": {
                    "sensitive": false,
                    "action": "create"
                },
                "AZURE_SERVICE_BUS_URI": {
                    "sensitive": true,
                    "action": "create"
                },
                "AZURE_TENANT_ID": {
                    "sensitive": false,
                    "action": "create"
                }
            },
            "type": "outputs"
        }
        "#;
        let outputs: TerraformPlanStep = serde_json::from_str(json).unwrap();
        assert_eq!(outputs.record_type, "outputs");
        assert_eq!(outputs.level, "info");
        assert_eq!(outputs.message, "Outputs: 25");
        assert_eq!(outputs.module, "terraform.ui");
        assert_eq!(outputs.timestamp, "2024-02-23T13:50:04.652705-06:00");
        let inner_outputs = outputs.outputs.unwrap();
        assert_eq!(*&inner_outputs.len(), 25);
        assert_eq!(
            *&inner_outputs.get("AZURE_AKS_CLUSTER_ID").unwrap().sensitive,
            false
        );
        assert_eq!(
            *&inner_outputs.get("AZURE_AKS_CLUSTER_ID").unwrap().action,
            "create"
        );
        assert_eq!(
            *&inner_outputs
                .get("AZURE_AKS_CLUSTER_NAME")
                .unwrap()
                .sensitive,
            false
        );
        assert_eq!(
            *&inner_outputs.get("AZURE_AKS_CLUSTER_NAME").unwrap().action,
            "create"
        );
    }
}
