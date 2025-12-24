//! 用户领域模型
//! 对齐 migrations/001_initial_schema.sql#users 表结构

use serde::{Deserialize, Serialize};

/// 用户领域模型
/// 字段对齐 migrations/001_initial_schema.sql#users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户 ID (TEXT PRIMARY KEY)
    pub id: String,
    /// 用户名 (TEXT NOT NULL UNIQUE)
    pub username: String,
    /// 密码哈希 (TEXT NOT NULL) - Argon2 PHC 字符串格式
    pub password_hash: String,
    /// 创建时间 (INTEGER NOT NULL) - Unix 毫秒时间戳
    pub created_at: i64,
    /// 更新时间 (INTEGER NOT NULL) - Unix 毫秒时间戳
    pub updated_at: i64,
}

impl User {
    /// 创建新用户实例
    pub fn new(id: String, username: String, password_hash: String, created_at: i64) -> Self {
        Self {
            id,
            username,
            password_hash,
            created_at,
            updated_at: created_at,
        }
    }
}
