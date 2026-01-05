use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct GenericConfig {
    pub variables: Vec<GenericInputVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct GenericInputVariable {
    pub name: String,
    pub value_type: GenericValueType,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub enum GenericValueType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "json")]
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct SaveGenericConfigRequest {
    pub variables: Vec<GenericInputVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct SaveGenericConfigResponse {
    pub generic_config: GenericConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct DeleteGenericConfigResponse {
    pub message: String,
}
