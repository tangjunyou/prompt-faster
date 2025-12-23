//! 后端配置唯一入口
//! 所有模块从此获取配置，不得直接读 env

use std::env;

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 数据库 URL
    pub database_url: String,
    /// 服务器地址
    pub server_host: String,
    /// 服务器端口
    pub server_port: u16,
    /// 日志级别
    pub log_level: String,
    /// 是否为开发模式
    pub is_dev: bool,
    /// CORS 允许的 Origins（逗号分隔）
    pub cors_origins: Vec<String>,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/prompt_faster.db?mode=rwc".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            is_dev: env::var("APP_ENV")
                .map(|v| v == "development")
                .unwrap_or(true),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }

    /// 获取完整的服务器地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
