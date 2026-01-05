//! 测试集（TestSet）领域模型

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::models::TestCase;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct TestSet {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub cases: Vec<TestCase>,
    /// 测试集维度的 Dify 变量配置（JSON 文本，字段结构见 Story 2.4）
    pub dify_config_json: Option<String>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}
