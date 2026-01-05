use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::infra::external::dify_client::DifyInputVariable;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct DifyConfig {
    pub target_prompt_variable: String,
    pub bindings: HashMap<String, DifyBinding>,
    pub parameters_snapshot: Option<Vec<DifyInputVariable>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct SaveDifyConfigRequest {
    pub target_prompt_variable: String,
    pub bindings: HashMap<String, DifyBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct SaveDifyConfigResponse {
    pub dify_config: DifyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "camelCase")]
pub struct DifyBinding {
    pub source: DifyBindingSource,
    pub value: Option<serde_json::Value>,
    pub input_key: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub enum DifyBindingSource {
    #[serde(rename = "fixed")]
    Fixed,
    #[serde(rename = "testCaseInput")]
    TestCaseInput,
}
