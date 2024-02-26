use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::output_tester::ResourceResult;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct AzureResourceChange {
    pub changes: Vec<AzureResourceChangeDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AzureResourceChangeType {
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
    pub after: Option<AzureResource>,
    pub before: Option<AzureResource>,
    #[serde(rename = "changeType")]
    pub change_type: AzureResourceChangeType,
    pub delta: Option<String>,
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "unsupportedReason")]
    pub unsupported_reason: Option<String>,
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

impl AzureResource {
    pub fn get_comparison_resource(&self) -> ResourceResult {
        let resource_type = match self.resource_type.as_str() {
            "Microsoft.Resources/resourceGroups" => "resource_group".to_string(),
            "Microsoft.ContainerService/managedClusters" => "kubernetes_cluster".to_string(),
            "Microsoft.Network/virtualNetworks" => "virtual_network".to_string(),
            "Microsoft.Network/publicIPAddresses" => "public_ip".to_string(),
            "Microsoft.Network/networkInterfaces" => "network_interface".to_string(),
            "Microsoft.Network/networkSecurityGroups" => "network_security_group".to_string(),
            "Microsoft.KeyVault/vaults" => "key_vault".to_string(),
            "Microsoft.Authorization/roleAssignments" => "role_assignment".to_string(),
            _ => self.resource_type.to_string(),
        };
        ResourceResult {
            resource_type: resource_type,
            resource_name: Some(self.name.clone()),
            provider: None,
            is_expected: None,
        }
    }
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
