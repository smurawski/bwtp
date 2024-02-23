use std::collections::HashMap;
use serde::{Serialize, Deserialize};



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
    location: String,
    name: String,
    tags: Option<HashMap<String, String>>,
    #[serde(rename = "type")]
    resource_type: String,
}

/*
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
*/
